use crate::{
    atomics::AtomicNum,
    error::{PromError, PromErrorKind, Result},
    histogram::HistogramCore,
    label::{valid_label_name, Label},
    registry::{Collectable, Descriptor},
};
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Write,
    hash::Hash,
    iter::{self, FromIterator},
    sync::atomic::AtomicU64,
};

// TODO: Optional fast hashers like fnv and fxhash
#[derive(Debug)]
pub struct Group<T, K: Key> {
    metrics: HashMap<K, T>,
}

impl<T, K: Key> Group<T, K> {
    pub(crate) fn new(metrics: HashMap<K, T>) -> Self {
        Self { metrics }
    }

    pub fn get(&self, key: K) -> &T {
        self.metrics
            .get(&key)
            .unwrap_or_else(|| panic!("The key value {} doesn't exist", key.key_name()))
    }

    pub fn try_get(&self, key: K) -> Option<&T> {
        self.metrics.get(&key)
    }
}

pub trait Key: Hash + Eq {
    fn key_name<'a>(&'a self) -> Cow<'a, str>;
}

impl<T> Key for T
where
    T: AsRef<str> + Hash + Eq,
{
    fn key_name<'a>(&'a self) -> Cow<'a, str> {
        Cow::Borrowed(self.as_ref())
    }
}

#[derive(Debug)]
pub struct CounterGroup<K: Key, Atomic: AtomicNum = AtomicU64> {
    group: Group<Atomic, K>,
    descriptor: Descriptor,
    bucket_label: Cow<'static, str>,
}

impl<K, Atomic> CounterGroup<K, Atomic>
where
    K: Key,
    Atomic: AtomicNum,
{
    pub fn new<N, H, L, V>(group_name: N, group_help: H, bucket_label: L, keys: V) -> Result<Self>
    where
        N: Into<Cow<'static, str>>,
        H: AsRef<str>,
        L: Into<Cow<'static, str>>,
        V: Iterator<Item = K>,
    {
        let bucket_label = bucket_label.into();
        if !valid_label_name(&bucket_label) {
            return Err(PromError::new(
                "Label name contains invalid characters",
                PromErrorKind::InvalidLabelName,
            ));
        }

        // TODO: Check for duplicates
        Ok(Self {
            group: Group::new(HashMap::from_iter(
                keys.zip(iter::from_fn(|| Some(Atomic::new()))),
            )),
            descriptor: Descriptor::new(group_name, group_help, Vec::new())?,
            bucket_label,
        })
    }

    pub fn inc(&self, key: K) {
        self.group.get(key).inc();
    }

    pub fn inc_by(&self, key: K, val: Atomic::Type) {
        self.group.get(key).inc_by(val);
    }

    pub fn set(&self, key: K, val: Atomic::Type) {
        self.group.get(key).set(val);
    }

    pub fn get(&self, key: K) -> Atomic::Type {
        self.group.get(key).get()
    }

    pub fn try_get(&self, key: K) -> Option<Atomic::Type> {
        self.group.try_get(key).map(|a| a.get())
    }

    pub fn clear(&self, key: K) {
        self.group.get(key).clear();
    }

    pub fn name(&self) -> &str {
        self.descriptor.name()
    }

    pub fn help(&self) -> &str {
        self.descriptor.help()
    }

    pub fn labels(&self) -> &[Label] {
        self.descriptor.labels()
    }
}

impl<K: Key, Atomic: AtomicNum> Collectable for &CounterGroup<K, Atomic> {
    fn encode_text<'a>(&'a self, buf: &mut String) -> Result<()> {
        writeln!(buf, "# HELP {} {}", self.name(), self.help())?;
        writeln!(buf, "# TYPE {} counter", self.name())?;

        for (bucket, value) in self.group.metrics.iter() {
            write!(
                buf,
                "{}{{{}={:?}",
                self.name(),
                self.bucket_label,
                bucket.key_name()
            )?;

            if !self.labels().is_empty() {
                write!(buf, ",")?;

                let mut labels = self.labels().iter();
                let last = labels.next_back();

                for label in labels {
                    write!(buf, "{}={:?},", label.name(), label.value())?;
                }

                if let Some(last) = last {
                    write!(buf, "{}={:?}", last.name(), last.value())?;
                }
            }

            write!(buf, "}} ")?;

            <Atomic as AtomicNum>::format(value.get(), buf, false)?;
            writeln!(buf)?;
        }

        Ok(())
    }

    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

#[derive(Debug)]
pub struct HistogramGroup<K: Key, Atomic: AtomicNum = AtomicU64> {
    group: Group<HistogramCore<Atomic>, K>,
    descriptor: Descriptor,
    bucket_label: Cow<'static, str>,
}

impl<K, Atomic> HistogramGroup<K, Atomic>
where
    K: Key,
    Atomic: AtomicNum,
{
    pub fn new<N, H, L, V, B>(
        group_name: N,
        group_help: H,
        bucket_label: L,
        keys: V,
        buckets: B,
    ) -> Result<Self>
    where
        N: Into<Cow<'static, str>>,
        H: AsRef<str>,
        L: Into<Cow<'static, str>>,
        V: Iterator<Item = K>,
        B: Iterator<Item = Atomic::Type>,
    {
        let bucket_label = bucket_label.into();
        if !valid_label_name(&bucket_label) {
            return Err(PromError::new(
                "Label name contains invalid characters",
                PromErrorKind::InvalidLabelName,
            ));
        }

        let buckets: Vec<Atomic::Type> = buckets.collect();

        // TODO: Check for duplicates
        Ok(Self {
            group: Group::new(HashMap::from_iter(
                keys.zip(iter::from_fn(|| Some(HistogramCore::new(buckets.clone())))),
            )),
            descriptor: Descriptor::new(group_name, group_help, Vec::new())?,
            bucket_label,
        })
    }

    pub fn get(&self, key: K) -> &HistogramCore<Atomic> {
        self.group.get(key)
    }

    pub fn try_get(&self, key: K) -> Option<&HistogramCore<Atomic>> {
        self.group.try_get(key)
    }

    pub fn clear(&self, key: K) {
        self.group.get(key).clear();
    }

    pub fn name(&self) -> &str {
        self.descriptor.name()
    }

    pub fn help(&self) -> &str {
        self.descriptor.help()
    }

    pub fn labels(&self) -> &[Label] {
        self.descriptor.labels()
    }
}

impl<K: Key, Atomic: AtomicNum> Collectable for &HistogramGroup<K, Atomic> {
    fn encode_text<'a>(&'a self, buf: &mut String) -> Result<()> {
        writeln!(buf, "# HELP {} {}", self.name(), self.help())?;
        writeln!(buf, "# TYPE {} histogram", self.name())?;

        let row = |buf: &mut String, name, bucket: &str| -> Result<()> {
            write!(
                buf,
                "{}_{}{{{}={:?}",
                self.name(),
                name,
                self.bucket_label,
                bucket,
            )?;

            if !self.labels().is_empty() {
                let mut labels = self.labels().iter();
                let last = labels.next_back();

                for label in labels {
                    write!(buf, ",{}={:?}", label.name(), label.value())?;
                }

                if let Some(last) = last {
                    write!(buf, "{}={:?}", last.name(), last.value())?;
                }
            }

            write!(buf, "}} ")?;

            Ok(())
        };

        for (bucket, histogram) in self.group.metrics.iter() {
            let bucket_name = bucket.key_name();

            row(buf, "sum", &bucket_name)?;
            Atomic::format(histogram.get_sum(), buf, false)?;
            writeln!(buf)?;

            row(buf, "count", &bucket_name)?;
            <AtomicU64 as AtomicNum>::format(histogram.get_count(), buf, false)?;
            writeln!(buf)?;

            for (i, bucket) in histogram.buckets.iter().enumerate() {
                write!(
                    buf,
                    "{}_bucket{{{}={:?},le=",
                    self.name(),
                    self.bucket_label,
                    &bucket_name,
                )?;
                Atomic::format(*bucket, buf, true)?;

                if !self.labels().is_empty() {
                    let mut labels = self.labels().iter();
                    let last = labels.next_back();

                    for label in labels {
                        write!(buf, ",{}={:?}", label.name(), label.value())?;
                    }

                    if let Some(last) = last {
                        write!(buf, "{}={:?}", last.name(), last.value())?;
                    }
                }

                write!(buf, "}} ")?;

                Atomic::format(histogram.values[i].get(), buf, false)?;
                writeln!(buf)?;
            }
        }

        Ok(())
    }

    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    enum GroupKey {
        A,
        B,
        C,
        D,
        E,
        F,
        G,
    }

    impl Key for GroupKey {
        fn key_name<'a>(&'a self) -> Cow<'a, str> {
            match self {
                Self::A => "a",
                Self::B => "b",
                Self::C => "c",
                Self::D => "d",
                Self::E => "e",
                Self::F => "f",
                Self::G => "g",
            }
            .into()
        }
    }

    #[test]
    fn counter_group() {
        let group: CounterGroup<GroupKey> = CounterGroup::new(
            "counters",
            "A group of counters",
            "group_key",
            vec![
                GroupKey::A,
                GroupKey::B,
                GroupKey::C,
                GroupKey::D,
                GroupKey::E,
                GroupKey::F,
                GroupKey::G,
            ]
            .into_iter(),
        )
        .unwrap();

        group.inc(GroupKey::A);
        assert_eq!(group.get(GroupKey::A), 1);
    }

    #[test]
    fn counter_group_strings() {
        let group: CounterGroup<&'static str> = CounterGroup::new(
            "counters",
            "A group of counters",
            "this_is_the_key",
            vec![
                "key_one",
                "key_two",
                "key_three",
                "key_four",
                "key_five",
                "key_six",
                "key_seven",
            ]
            .into_iter(),
        )
        .unwrap();

        group.inc("key_one");
        assert_eq!(group.get("key_one"), 1);
    }

    #[test]
    fn histogram_group() {
        let group: HistogramGroup<&'static str> = HistogramGroup::new(
            "histogram_group",
            "It's a group of histograms",
            "histogram_bucket",
            vec!["bucket1", "bucket2", "bucket3", "bucket4"].into_iter(),
            vec![1u64, 2, 3, 4].into_iter(),
        )
        .unwrap();

        group.get("bucket1").observe(4);
        group.get("bucket2").observe(3);
        group.get("bucket3").observe(2);
        group.get("bucket4").observe(1);

        assert_eq!(group.get("bucket1").values(), vec![0, 0, 0, 1]);
        assert_eq!(group.get("bucket2").values(), vec![0, 0, 1, 0]);
        assert_eq!(group.get("bucket3").values(), vec![0, 1, 0, 0]);
        assert_eq!(group.get("bucket4").values(), vec![1, 0, 0, 0]);
    }
}

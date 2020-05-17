use crate::{
    atomics::AtomicF64,
    counter::{Counter, FloatCounter, IntCounter, UintCounter},
    label::{Label, Labeled, Metric},
};
use std::{
    fmt::{Result, Write},
    sync::atomic::{AtomicI64, AtomicU64},
};

pub trait Exportable {
    fn export(&self, f: &mut impl Write) -> Result;
}

impl<T: Exportable + Metric> Exportable for Labeled<T> {
    fn export(&self, f: &mut impl Write) -> Result {
        write!(
            f,
            "# HELP {}\n# TYPE {}\n{}",
            self.description(),
            self.metric_kind(),
            self.name()
        )?;

        if !self.labels().is_empty() {
            write!(f, "{{")?;

            for (i, Label { label, value }) in self.labels().iter().enumerate() {
                write!(
                    f,
                    "{}={:?}{}",
                    &*label,
                    &*value,
                    if i != self.labels().len() - 1 {
                        ","
                    } else {
                        ""
                    },
                )?;
            }

            write!(f, "}} ")?;
        } else {
            write!(f, " ")?
        }

        Exportable::export(self.data(), f)?;
        writeln!(f)
    }
}

impl Exportable for Counter<AtomicF64> {
    fn export(&self, f: &mut impl Write) -> Result {
        let val = self.get();
        match val {
            val if val.is_nan() => write!(f, "Nan"),
            val if val.is_infinite() && val.is_sign_positive() => write!(f, "+Inf"),
            val if val.is_infinite() && val.is_sign_negative() => write!(f, "-Inf"),

            val => write!(f, "{:?}", val),
        }
    }
}

impl Exportable for Counter<AtomicU64> {
    fn export(&self, f: &mut impl Write) -> Result {
        write!(f, "{}", self.get())
    }
}

impl Exportable for Counter<AtomicI64> {
    fn export(&self, f: &mut impl Write) -> Result {
        write!(f, "{}", self.get())
    }
}

macro_rules! export_inner {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Exportable for $ty {
                fn export(&self, f: &mut impl Write) -> Result {
                    Exportable::export(&self.0, f)
                }
            }
        )*
    };
}

export_inner!(UintCounter, FloatCounter, IntCounter);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_uint_counter() {
        let counter = UintCounter::new("uint_counter", "Counts things").unwrap();
        counter.inc();

        let expected = "# HELP Counts things\n# TYPE counter\nuint_counter 1\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);

        let counter = UintCounter::new("uint_counter", "Counts things")
            .unwrap()
            .with_labels(vec![Label::new("some_label", "some_value").unwrap()]);
        counter.inc();

        let expected =
            "# HELP Counts things\n# TYPE counter\nuint_counter{some_label=\"some_value\"} 1\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);

        let counter = UintCounter::new("uint_counter", "Counts things")
            .unwrap()
            .with_labels(vec![
                Label::new("some_label", "some_value").unwrap(),
                Label::new("another_label", "another_value").unwrap(),
            ]);
        counter.inc();

        let expected =
            "# HELP Counts things\n# TYPE counter\nuint_counter{some_label=\"some_value\",another_label=\"another_value\"} 1\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);
    }

    #[test]
    fn export_float_counter() {
        let counter = FloatCounter::new("float_counter", "Counts things").unwrap();
        counter.inc();

        let expected = "# HELP Counts things\n# TYPE counter\nfloat_counter 1.0\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);

        let counter = FloatCounter::new("float_counter", "Counts things")
            .unwrap()
            .with_labels(vec![Label::new("some_label", "some_value").unwrap()]);
        counter.inc();

        let expected =
            "# HELP Counts things\n# TYPE counter\nfloat_counter{some_label=\"some_value\"} 1.0\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);

        let counter = FloatCounter::new("float_counter", "Counts things")
            .unwrap()
            .with_labels(vec![
                Label::new("some_label", "some_value").unwrap(),
                Label::new("another_label", "another_value").unwrap(),
            ]);
        counter.inc();

        let expected =
            "# HELP Counts things\n# TYPE counter\nfloat_counter{some_label=\"some_value\",another_label=\"another_value\"} 1.0\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);

        let counter = FloatCounter::new("float_counter", "Counts things")
            .unwrap()
            .with_labels(vec![Label::new("some_label", "some_value").unwrap()]);
        counter.set(f64::NAN);

        let expected =
            "# HELP Counts things\n# TYPE counter\nfloat_counter{some_label=\"some_value\"} Nan\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);

        let counter = FloatCounter::new("float_counter", "Counts things")
            .unwrap()
            .with_labels(vec![Label::new("some_label", "some_value").unwrap()]);
        counter.set(f64::INFINITY);

        let expected =
            "# HELP Counts things\n# TYPE counter\nfloat_counter{some_label=\"some_value\"} +Inf\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);

        let counter = FloatCounter::new("float_counter", "Counts things")
            .unwrap()
            .with_labels(vec![Label::new("some_label", "some_value").unwrap()]);
        counter.set(f64::NEG_INFINITY);

        let expected =
            "# HELP Counts things\n# TYPE counter\nfloat_counter{some_label=\"some_value\"} -Inf\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);
    }

    #[test]
    fn export_int_counter() {
        let counter = IntCounter::new("int_counter", "Counts things").unwrap();
        counter.inc();

        let expected = "# HELP Counts things\n# TYPE counter\nint_counter 1\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);

        let counter = IntCounter::new("int_counter", "Counts things")
            .unwrap()
            .with_labels(vec![Label::new("some_label", "some_value").unwrap()]);
        counter.inc();

        let expected =
            "# HELP Counts things\n# TYPE counter\nint_counter{some_label=\"some_value\"} 1\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);

        let counter = IntCounter::new("int_counter", "Counts things")
            .unwrap()
            .with_labels(vec![
                Label::new("some_label", "some_value").unwrap(),
                Label::new("another_label", "another_value").unwrap(),
            ]);
        counter.inc();

        let expected = "# HELP Counts things\n# TYPE counter\nint_counter{some_label=\"some_value\",another_label=\"another_value\"} 1\n";
        let mut exported = String::new();
        Exportable::export(&counter, &mut exported).unwrap();

        assert_eq!(expected, exported);
    }
}

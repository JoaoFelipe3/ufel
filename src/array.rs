use std::{
    fmt,
    hash::{Hash, Hasher},
};

use ecow::EcoVec;

use crate::{cowslice::CowSlice, Form, Ufel, UfelResult};

#[derive(Clone)]
pub struct Array<T = f64> {
    pub form: Form,
    pub data: CowSlice<T>,
}

impl<T> Default for Array<T> {
    fn default() -> Self {
        Self {
            form: Form::empty_list(),
            data: CowSlice::new(),
        }
    }
}

impl<T: Clone> Array<T> {
    pub fn new(form: Form, data: CowSlice<T>) -> Self {
        Self { form, data }
    }
    pub fn scalar(data: T) -> Self {
        Self {
            form: Form::scalar(),
            data: CowSlice::from_elem(data, 1),
        }
    }
    #[track_caller]
    pub fn validate_shape(&self) {
        self.form.validate();
        #[cfg(debug_assertions)]
        assert_eq!(
            self.form.elems(),
            self.data.len(),
            "Form is {:?} but data has {} elements",
            self.form,
            self.data.len()
        );
    }
    pub fn from_row_arrays(rows: impl IntoIterator<Item = Self>, rt: &Ufel) -> UfelResult<Self> {
        let mut iter = rows.into_iter();
        let Some(mut arr) = iter.next() else {
            return Ok(Array::default());
        };
        if arr.form.is_normal() {
            let mut new_len = 1;
            for row in iter {
                if row.form != arr.form {
                    return Err(rt.error(format!(
                        "Cannot create array with different row forms {:?} and {:?}",
                        arr.form, row.form
                    )));
                }
                arr.data.extend_from_cowslice(row.data);
                new_len += 1;
            }
            arr.form.fix();
            arr.form[0][0] = new_len;
            arr.validate_shape();
            Ok(arr)
        } else {
            todo!("non-normal array creation")
        }
    }
}

impl<T: Element> Array<T> {}

impl<T: Element> PartialEq for Array<T> {
    fn eq(&self, other: &Self) -> bool {
        self.form == other.form
            && (self.data.iter())
                .zip(other.data.iter())
                .all(|(a, b)| a.array_eq(b))
    }
}

impl<T: Element> Eq for Array<T> {}

impl<T: Element> Hash for Array<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.form.hash(state);
        self.data.iter().for_each(|elem| elem.array_hash(state));
    }
}

pub trait Element: fmt::Debug + Clone + Send + Sync + 'static {
    fn array_eq(&self, other: &Self) -> bool;
    fn array_hash<H: Hasher>(&self, state: &mut H);
}

impl Element for f64 {
    fn array_eq(&self, other: &Self) -> bool {
        self == other || self.is_nan() && other.is_nan()
    }
    fn array_hash<H: Hasher>(&self, state: &mut H) {
        if self.is_nan() {
            f64::NAN.to_bits().hash(state);
        } else {
            self.to_bits().hash(state);
        }
    }
}

impl<T: Clone> From<T> for Array<T> {
    fn from(data: T) -> Self {
        Self::scalar(data)
    }
}

impl<T: Clone> From<EcoVec<T>> for Array<T> {
    fn from(data: EcoVec<T>) -> Self {
        Self {
            form: data.len().into(),
            data: data.into(),
        }
    }
}

impl<T: Clone> From<CowSlice<T>> for Array<T> {
    fn from(data: CowSlice<T>) -> Self {
        Self {
            form: data.len().into(),
            data,
        }
    }
}

impl<T: Clone, const N: usize> From<[T; N]> for Array<T> {
    fn from(data: [T; N]) -> Self {
        Self {
            form: data.len().into(),
            data: data.into(),
        }
    }
}

impl<T: Copy> From<&[T]> for Array<T> {
    fn from(data: &[T]) -> Self {
        Self {
            form: data.len().into(),
            data: data.into(),
        }
    }
}

impl<T: Clone> FromIterator<T> for Array<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let data = CowSlice::from_iter(iter);
        Self {
            form: data.len().into(),
            data,
        }
    }
}

impl From<usize> for Array {
    fn from(n: usize) -> Self {
        Self::scalar(n as f64)
    }
}

impl From<&[usize]> for Array {
    fn from(value: &[usize]) -> Self {
        value.iter().map(|&n| n as f64).collect()
    }
}

impl From<Form> for Array {
    fn from(form: Form) -> Self {
        let mut arr: Array = form.dims().into();
        arr.form = Form::from([form.row_rank(), form.col_rank()]);
        arr
    }
}

impl<T: fmt::Debug> fmt::Debug for Array<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.form.is_scalar() {
            self.data[0].fmt(f)
        } else {
            write!(f, "[")?;
            if !self.form.is_list() {
                write!(f, "{:?} ", self.form)?;
            }
            for (i, elem) in self.data.iter().enumerate() {
                if i > 0 {
                    write!(f, " ")?;
                }
                elem.fmt(f)?;
            }
            write!(f, "]")
        }
    }
}

impl<T: fmt::Display> fmt::Display for Array<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.form.is_scalar() {
            self.data[0].fmt(f)
        } else {
            write!(f, "[")?;
            if !self.form.is_list() {
                write!(f, "{:?} ", self.form)?;
            }
            for (i, elem) in self.data.iter().enumerate() {
                if i > 0 {
                    write!(f, " ")?;
                }
                elem.fmt(f)?;
            }
            write!(f, "]")
        }
    }
}

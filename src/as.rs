use core::marker::PhantomData;

pub struct Conv<Frm: ?Sized, To: ?Sized>(PhantomData<(Box<Frm>, Box<To>)>);

impl<Frm: ?Sized, To: ?Sized> Default for Conv<Frm, To> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub trait ExtractRef {
    type Frm;
    type To;

    fn __extract_ref(&self, frm: Self::Frm) -> Self::To;
}

impl<'a, T: ?Sized> ExtractRef for &Conv<&'a T, T> {
    type Frm = &'a T;
    type To = &'a T;

    fn __extract_ref(&self, frm: Self::Frm) -> Self::To {
        frm
    }
}

impl<'a, Frm: ?Sized + AsRef<To>, To: 'a + ?Sized> ExtractRef for Conv<&'a Frm, To> {
    type Frm = &'a Frm;
    type To = &'a To;

    fn __extract_ref(&self, frm: Self::Frm) -> Self::To {
        frm.as_ref()
    }
}

impl<'a, T: ?Sized> ExtractRef for &Conv<&'a mut T, T> {
    type Frm = &'a mut T;
    type To = &'a mut T;

    fn __extract_ref(&self, frm: Self::Frm) -> Self::To {
        frm
    }
}

impl<'a, Frm: ?Sized + AsMut<To>, To: ?Sized + 'a> ExtractRef
    for Conv<&'a mut Frm, To>
{
    type Frm = &'a mut Frm;
    type To = &'a mut To;

    fn __extract_ref(&self, frm: Self::Frm) -> Self::To {
        frm.as_mut()
    }
}

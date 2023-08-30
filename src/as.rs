use core::marker::PhantomData;

pub struct Conv<Frm, To>(PhantomData<(Frm, To)>);

pub trait GetRef {
    type Frm;
    type To;

    fn get_ref(&self, frm: Self::Frm) -> Self::To;
}

impl<'a, T> GetRef for &Conv<&'a T, T> {
    type Frm = &'a T;
    type To = &'a T;

    fn get_ref(&self, frm: Self::Frm) -> Self::To {
        frm
    }
}

impl<'a, Frm: AsRef<To>, To: 'a> GetRef for Conv<&'a Frm, To> {
    type Frm = &'a Frm;
    type To = &'a To;

    fn get_ref(&self, frm: Self::Frm) -> Self::To {
        frm.as_ref()
    }
}

impl<'a, T> GetRef for &Conv<&'a mut T, T> {
    type Frm = &'a mut T;
    type To = &'a mut T;

    fn get_ref(&self, frm: Self::Frm) -> Self::To {
        frm
    }
}

impl<'a, Frm: AsMut<To>, To: 'a> GetRef for Conv<&'a mut Frm, To> {
    type Frm = &'a mut Frm;
    type To = &'a mut To;

    fn get_ref(&self, frm: Self::Frm) -> Self::To {
        frm.as_mut()
    }
}

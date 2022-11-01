use super::dihedral;
use std::ops::Mul;

//translations use "x = right", "y = up" coordinates

//when lhs is a "small" transformation, multiplication can fail (specifically when rhs has a non-even translation)
//inverting a "large" transformation can also fail (if it has a non-even translation)
//large cannot be multiplied by large and likewise with small. There is no type that incorporates arbitrary scaling here.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct EvenD8Translation {
	dihedral: dihedral::EvenD8,
	translation: (isize,isize),
}

impl EvenD8Translation {
	
	pub const fn new(flip: bool, rot: i8, translation: (isize, isize)) -> EvenD8Translation {
		EvenD8Translation {
			dihedral: dihedral::EvenD8::new(flip, rot),
			translation,
		}
	}
	
	pub fn inv(self) -> EvenD8Translation {
		let inverse_dihedral = self.dihedral.inv();
		EvenD8Translation {
			dihedral: inverse_dihedral,
			translation: inverse_dihedral.apply((-self.translation.0, -self.translation.1))
		}
	}
	
	pub fn get_translation(self) -> (isize, isize) {
		self.translation
	}
	
	pub fn get_dihedral(self) -> dihedral::EvenD8 {
		self.dihedral
	}
}

//even * even
impl Mul<EvenD8Translation> for EvenD8Translation {
	type Output = EvenD8Translation;
	
	fn mul(self, rhs: EvenD8Translation) -> Self::Output {
		let conjugated_translation = self.dihedral.apply(rhs.translation);
		EvenD8Translation {
			dihedral: self.dihedral * rhs.dihedral,
			translation: (self.translation.0 + conjugated_translation.0, self.translation.1 + conjugated_translation.1)
		}
	}
}



//These come with a scale down by sqrt(2) preapplied at the start
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct OddD8TranslationSmall {
	dihedral: dihedral::OddD8,
	translation: (isize, isize),
}

//These come with a scale up by sqrt(2) preapplied at the start
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct OddD8TranslationLarge {
	dihedral: dihedral::OddD8,
	translation: (isize, isize),
}

impl OddD8TranslationSmall {
	pub const fn new(flip: bool, rot: i8, translation: (isize, isize)) -> OddD8TranslationSmall {
		OddD8TranslationSmall {
			dihedral: dihedral::OddD8::new(flip, rot),
			translation,
		}
	}
	
	pub fn inv(self) -> OddD8TranslationLarge {
		//this is a large version rather than a small version like self
		let inverse_dihedral = self.dihedral.inv();
		OddD8TranslationLarge{
			dihedral: inverse_dihedral,
			translation: inverse_dihedral.apply_as_large((-self.translation.0, -self.translation.1))
		}
	}
	
	pub fn get_translation(self) -> (isize, isize) {
		self.translation
	}
	
	pub fn get_dihedral(self) -> dihedral::OddD8 {
		self.dihedral
	}
}

impl OddD8TranslationLarge {
	pub const fn new(flip: bool, rot: i8, translation: (isize, isize)) -> OddD8TranslationLarge {
		OddD8TranslationLarge {
			dihedral: dihedral::OddD8::new(flip, rot),
			translation,
		}
	}
	
	//option because it is possible for odd translations to have non-integer translation inverses. Only integer translations are allowed in the output type.
	pub fn inv(self) -> Option<OddD8TranslationSmall> {
		//this is a small version rather than a large version like self
		let inverse_dihedral = self.dihedral.inv();
		Some(OddD8TranslationSmall{
			dihedral: inverse_dihedral,
			translation: inverse_dihedral.apply_as_small((-self.translation.0, -self.translation.1))?
		})
	}
	
	pub fn get_translation(self) -> (isize, isize) {
		self.translation
	}
	
	pub fn get_dihedral(self) -> dihedral::OddD8 {
		self.dihedral
	}
}

//odd * odd (no large*large or small*small allowed)
impl Mul<OddD8TranslationSmall> for OddD8TranslationLarge {
	type Output = EvenD8Translation;
	
	fn mul(self, rhs: OddD8TranslationSmall) -> Self::Output {
		let conjugated_translation = self.dihedral.apply_as_large(rhs.translation);
		EvenD8Translation {
			dihedral: self.dihedral * rhs.dihedral,
			translation: (self.translation.0 + conjugated_translation.0, self.translation.1 + conjugated_translation.1)
		}
	}
}

impl Mul<OddD8TranslationLarge> for OddD8TranslationSmall {
	type Output = Option<EvenD8Translation>;
	
	fn mul(self, rhs: OddD8TranslationLarge) -> Self::Output {
		let conjugated_translation = self.dihedral.apply_as_small(rhs.translation)?;
		Some(EvenD8Translation {
			dihedral: self.dihedral * rhs.dihedral,
			translation: (self.translation.0 + conjugated_translation.0, self.translation.1 + conjugated_translation.1)
		})
	}
}




//even * odd
impl Mul<OddD8TranslationSmall> for EvenD8Translation {
	type Output = OddD8TranslationSmall;
	
	fn mul(self, rhs: OddD8TranslationSmall) -> Self::Output {
		let conjugated_translation = self.dihedral.apply(rhs.translation);
		OddD8TranslationSmall {
			dihedral: self.dihedral * rhs.dihedral,
			translation: (self.translation.0 + conjugated_translation.0, self.translation.1 + conjugated_translation.1)
		}
	}
}

impl Mul<OddD8TranslationLarge> for EvenD8Translation {
	type Output = OddD8TranslationLarge;
	
	fn mul(self, rhs: OddD8TranslationLarge) -> Self::Output {
		let conjugated_translation = self.dihedral.apply(rhs.translation);
		OddD8TranslationLarge {
			dihedral: self.dihedral * rhs.dihedral,
			translation: (self.translation.0 + conjugated_translation.0, self.translation.1 + conjugated_translation.1)
		}
	}
}


//odd * even
impl Mul<EvenD8Translation> for OddD8TranslationLarge {
	type Output = OddD8TranslationLarge;
	
	fn mul(self, rhs: EvenD8Translation) -> Self::Output {
		let conjugated_translation = self.dihedral.apply_as_large(rhs.translation);
		OddD8TranslationLarge {
			dihedral: self.dihedral * rhs.dihedral,
			translation: (self.translation.0 + conjugated_translation.0, self.translation.1 + conjugated_translation.1)
		}
	}
}

impl Mul<EvenD8Translation> for OddD8TranslationSmall {
	type Output = Option<OddD8TranslationSmall>;
	
	fn mul(self, rhs: EvenD8Translation) -> Self::Output {
		let conjugated_translation = self.dihedral.apply_as_small(rhs.translation)?;
		Some(OddD8TranslationSmall {
			dihedral: self.dihedral * rhs.dihedral,
			translation: (self.translation.0 + conjugated_translation.0, self.translation.1 + conjugated_translation.1)
		})
	}
}


pub const IDENTITY: EvenD8Translation = EvenD8Translation{
	dihedral: dihedral::IDENTITY,
	translation: (0,0)
};
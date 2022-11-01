use std::ops::Mul;

//note: the even and odd rotation amounts are handled seperately because the odd ones tend to come with a pesky sqrt(2) scale
//the even ones are essentially just D4, but the odd ones are a bit more annoying
//they are represented as "flip? first, then do rot quarter turns"
//intrinsic: rot is 0, 1, 2, or 3.
//the remainder operator is trash, so &3 is used instead of %4
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct EvenD8 {
	flip: bool,
	rot: i8,
}

impl EvenD8 {
	pub const fn new(flip: bool, rot: i8) -> EvenD8 {
		EvenD8 {
			flip,
			rot: rot.rem_euclid(4),
		}
	}

	pub fn flipped(self) -> bool {self.flip}
	pub fn rot(self) -> i8 {self.rot}
	
	pub fn inv(self) -> EvenD8 {
		EvenD8 {
			flip: self.flip,
			rot: ( if self.flip {self.rot} else {-self.rot} ).rem_euclid(4),
		}
	}
	
	pub fn apply(self, point: (isize, isize)) -> (isize, isize) {
		//flip
		let point = if self.flip {
			(point.0, -point.1)
		} else {point};
		//rotate 1/4
		let point = if self.rot&1 == 1 {
			(-point.1, point.0)
		} else {point};
		//rotate 1/2
		let point = if self.rot >= 2 {
			(-point.0, -point.1)
		} else {point};
		
		point
	}
}

impl Mul<EvenD8> for EvenD8 {
	type Output = EvenD8;
	
	fn mul(self, rhs: EvenD8) -> Self::Output {
		if self.flip {
			EvenD8 {
				flip: !rhs.flip,
				rot: (self.rot - rhs.rot).rem_euclid(4),
			}
		} else {
			EvenD8 {
				flip: rhs.flip,
				rot: (self.rot + rhs.rot).rem_euclid(4),
			}
		}
	}
}


//The <rot=0> rotation is supposed to be "anticlockwise an eighth turn", and it's quarter turns from there.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct OddD8 {
	flip: bool,
	rot: i8,
}

impl OddD8 {
	pub const fn new(flip: bool, rot: i8) -> OddD8 {
		OddD8 {
			flip,
			rot: rot.rem_euclid(4),
		}
	}

	pub fn flipped(self) -> bool {self.flip}
	pub fn rot(self) -> i8 {self.rot}
	
	pub fn inv(self) -> OddD8 {
		OddD8 {
			flip: self.flip,
			rot: if self.flip {self.rot} else {3 - self.rot} //cool!
		}
	}
	
	pub fn apply_as_large(self, point: (isize, isize)) -> (isize, isize) {
		//flip (commutes with scale so can happen first)
		let point = if self.flip {
			(point.0, -point.1)
		} else {point};
		//rotate 1/8 and scale sqrt 2
		let point = (point.0 - point.1, point.0 + point.1);
		//rotate 1/4
		let point = if self.rot.rem_euclid(2) == 1 {
			(-point.1, point.0)
		} else {point};
		//rotate 1/2
		let point = if self.rot >= 2 {
			(-point.0, -point.1)
		} else {point};
		
		point
	}
	
	//option because non-even positions will get sent to non-integers
	pub fn apply_as_small(self, point: (isize, isize)) -> Option<(isize, isize)> {
		let point = self.apply_as_large(point);
		if point.0.rem_euclid(2) != 0 || point.1.rem_euclid(2) != 0 {
			//non-even position
			None
		} else {
			//same as large, but the scale is different
			Some((point.0.div_euclid(2), point.1.div_euclid(2)))
		}
	}
}

//Warning: this multiplication should be done in a way that incorporates some kind of doubling or halfing elsewhere
impl Mul<OddD8> for OddD8 {
	type Output = EvenD8;
	
	fn mul(self, rhs: OddD8) -> Self::Output {
		if self.flip {
			EvenD8 {
				flip: !rhs.flip,
				rot: (self.rot - rhs.rot).rem_euclid(4),
			}
		} else {
			EvenD8 {
				flip: rhs.flip,
				rot: (self.rot + rhs.rot + 1).rem_euclid(4),
			}
		}
	}
}





impl Mul<OddD8> for EvenD8 {
	type Output = OddD8;
	
	fn mul(self, rhs: OddD8) -> Self::Output {
		if self.flip {
			OddD8 {
				flip: !rhs.flip,
				rot: (self.rot - rhs.rot - 1).rem_euclid(4),
			}
		} else {
			OddD8 {
				flip: rhs.flip,
				rot: (self.rot + rhs.rot).rem_euclid(4),
			}
		}
	}
}


impl Mul<EvenD8> for OddD8 {
	type Output = OddD8;
	
	fn mul(self, rhs: EvenD8) -> Self::Output {
		if self.flip {
			OddD8 {
				flip: !rhs.flip,
				rot: (self.rot - rhs.rot).rem_euclid(4),
			}
		} else {
			OddD8 {
				flip: rhs.flip,
				rot: (self.rot + rhs.rot).rem_euclid(4),
			}
		}
	}
}


//if ever you want it I guess
pub const IDENTITY: EvenD8 = EvenD8{
	flip: false,
	rot: 0,
};
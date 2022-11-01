use std::ops::{Mul, Add, Sub, Div};

pub fn to_srgb(v: f64) -> f64 {
	if v <= 0.0031308 {
		12.92 * v
	} else {
		1.055 * f64::powf(v, 1.0/2.4) - 0.055
	}
}

pub fn from_srgb(v: f64) -> f64 {
	if v <= 0.04045 {
		v / 12.92
	} else {
		f64::powf((v + 0.055) / 1.055, 2.4)
	}
}

//alpha premultiplied
#[derive(Clone, Copy)]
pub struct LinearCol {
	r: f64,
	g: f64,
	b: f64,
	tau: f64,    //not an alpha channel, but rather a "tau channel"
	             //represents transmission rather than absorbtion
				 //tau = 1 - alpha
				 //it's a wonder that this isn't how it works elsewhere, considering how natural a transmission-emission system seems
}

impl LinearCol {
	pub fn new(r: f64, g: f64, b: f64, tau: f64) -> LinearCol { LinearCol{r, g, b, tau} }

	//WARNING: alpha premultiplied
	pub fn r_lin(self) -> f64 {self.r}
	pub fn g_lin(self) -> f64 {self.g}
	pub fn b_lin(self) -> f64 {self.b}
	pub fn y_lin(self) -> f64 {0.2162 * self.r + 0.7152 * self.g + 0.0722 * self.b}
	
	pub fn grayscale(self) -> u8 {
		let alpha = 1.0 - self.tau;
		let y_normalized = self.y_lin() / alpha; //fails if fully transparent, but what else is new?
		let y_srgb = to_srgb(y_normalized);
		(y_srgb * 256.0) as u8
	}
	
	pub fn rgb(self) -> (u8, u8, u8) {
		let alpha = 1.0 - self.tau;
		let r_normalized = self.r / alpha;
		let g_normalized = self.g / alpha;
		let b_normalized = self.b / alpha;
		let r_srgb = to_srgb(r_normalized);
		let g_srgb = to_srgb(g_normalized);
		let b_srgb = to_srgb(b_normalized);
		(
			(r_srgb * 256.0) as u8,
			(g_srgb * 256.0) as u8,
			(b_srgb * 256.0) as u8,
		)
	}
	
	pub fn rgba_separate(self) -> (u8, u8, u8, u8) {
		let alpha = 1.0 - self.tau;

		let r_normalized;
		let g_normalized;
		let b_normalized;
		if alpha != 0.0 {
			r_normalized = self.r / alpha;
			g_normalized = self.g / alpha;
			b_normalized = self.b / alpha;
		} else {
			r_normalized = 0.0;
			g_normalized = 0.0;
			b_normalized = 0.0;
		}


		let r_srgb = to_srgb(r_normalized);
		let g_srgb = to_srgb(g_normalized);
		let b_srgb = to_srgb(b_normalized);
		(
			(r_srgb * 256.0) as u8,
			(g_srgb * 256.0) as u8,
			(b_srgb * 256.0) as u8,
			(alpha  * 256.0) as u8,
		)
	}
	
	pub fn rgba_premul(self) -> (u8, u8, u8, u8) {
		//never mind, druid's rgba_premul is jank
		//it somehow has the worst of both worlds: it is gamma compressed AND deals in alpha or something
		let alpha = 1.0 - self.tau;
		(
			(self.r * 256.0) as u8,
			(self.g * 256.0) as u8,
			(self.b * 256.0) as u8,
			(alpha  * 256.0) as u8,
		)
	}
}

//I had the option to treat users of this module like babies and not give them any of the following methods, but I didn't.
//It's up to you to make sure these are only used for weighted averages.
//Or perhaps you actually want to treat them as vectors.
impl Add<LinearCol> for LinearCol {
	type Output = LinearCol;
	
	fn add(self, rhs: LinearCol) -> Self::Output {
		LinearCol{
			r: (self.r + rhs.r) as f64,
			g: (self.g + rhs.g) as f64,
			b: (self.b + rhs.b) as f64,
			tau: (self.tau + rhs.tau) as f64,
		}
	}
}

impl Mul<f64> for LinearCol {
	
	type Output = LinearCol;
	
	fn mul(self, rhs: f64) -> Self::Output {
		LinearCol{
			r: self.r * rhs,
			g: self.g * rhs,
			b: self.b * rhs,
			tau: self.tau * rhs,
		}
	}
}

impl Mul<LinearCol> for f64 {
	
	type Output = LinearCol;
	
	fn mul(self, rhs: LinearCol) -> Self::Output {
		LinearCol{
			r: self * rhs.r,
			g: self * rhs.g,
			b: self * rhs.b,
			tau: self * rhs.tau,
		}
	}
}

impl Sub<LinearCol> for LinearCol {
	type Output = LinearCol;
	
	fn sub(self, rhs: LinearCol) -> Self::Output {
		LinearCol {
			r: (self.r - rhs.r) as f64,
			g: (self.g - rhs.g) as f64,
			b: (self.b - rhs.b) as f64,
			tau: (self.tau - rhs.tau) as f64,
		}
	}
}

impl Div<f64> for LinearCol {
	
	type Output = LinearCol;
	
	fn div(self, rhs: f64) -> Self::Output {
		LinearCol{
			r: self.r / rhs,
			g: self.g / rhs,
			b: self.b / rhs,
			tau: self.tau / rhs,
		}
	}
}
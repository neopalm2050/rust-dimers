use std::{rc::Rc, borrow::Borrow};

use super::colour_format::LinearCol;

//CRITICAL (no slow impls allowed). Idea: big Rc<TreeNode> things should also be fast.
pub trait SplitInterval {
	fn get(&self) -> LinearCol;
	fn split(&self) -> (Box<dyn SplitInterval>, Box<dyn SplitInterval>);
}

//a colour can be interpreted as a constant interval
#[derive(Clone, Copy)]
pub struct ConstantInterval (LinearCol);

impl From<LinearCol> for ConstantInterval {
	fn from(col: LinearCol) -> Self {
		ConstantInterval(col)
	}
}

impl SplitInterval for ConstantInterval {
	fn get(&self) -> LinearCol {
		self.0
	}
	
	fn split(&self) -> (Box<dyn SplitInterval>, Box<dyn SplitInterval>) {
		(Box::new(*self), Box::new(*self))
	}
}


//an interval can be coloured by a function (from [0,1] to colour space)
//usually use a & or Rc to a Fn, as those clone easily
pub struct FunctionInterval<F> where F: Fn(f64) -> LinearCol + Clone + 'static {
	func: F,
	start: f64,
	end: f64,
}

impl<F: Fn(f64) -> LinearCol + Clone + 'static> From<F> for FunctionInterval<F> {
	fn from(func: F) -> FunctionInterval<F> {
		FunctionInterval{
			func,
			start: 0.0,
			end: 1.0,
		}
	}
}

impl<F: Fn(f64) -> LinearCol + Clone + 'static> SplitInterval for FunctionInterval<F> {
	fn get(&self) -> LinearCol {
		let middle = (self.start + self.end) / 2.0;
		(self.func)(middle)
	}
	
	fn split(&self) -> (Box<dyn SplitInterval>, Box<dyn SplitInterval>) {
		let middle = (self.start + self.end) / 2.0;
		(
			Box::new(FunctionInterval {
				func: self.func.clone(),
				start: self.start,
				end: middle,
			}),
			Box::new(FunctionInterval {
				func: self.func.clone(),
				start: middle,
				end: self.end,
			})
		)
	}
}

//first time ?Sized has come up in my experience
impl<T: SplitInterval + ?Sized> SplitInterval for Rc<T> {
	fn get(&self) -> LinearCol {
		let reference: &T = self.borrow();
		reference.get()
	}

	fn split(&self) -> (Box<dyn SplitInterval>, Box<dyn SplitInterval>) {
		let reference: &T = self.borrow();
		reference.split()
	}
}

pub struct TreeInterval {
	colour: LinearCol,
	left_child: Rc<dyn SplitInterval>,
	right_child: Rc<dyn SplitInterval>,
}

impl SplitInterval for TreeInterval {
	fn get(&self) -> LinearCol {
		self.colour
	}

	fn split(&self) -> (Box<dyn SplitInterval>, Box<dyn SplitInterval>) {
		(Box::new(self.left_child.clone()), Box::new(self.right_child.clone()))
	}
}

impl TreeInterval {
	pub fn new(colour: LinearCol, left_child: Rc<dyn SplitInterval>, right_child: Rc<dyn SplitInterval>) -> TreeInterval {
		TreeInterval{
			colour,
			left_child,
			right_child,
		}
	}
}

//import path from file into a closure?

use std::iter::zip;

use super::colour_format::LinearCol;
use super::dihedral_translation::{EvenD8Translation};
use super::interval_colouring::SplitInterval;

pub struct Triangle {
	required: Vec<Box<dyn SplitInterval>>
}

impl Triangle {

	//takes an EvenD8Translation and a colouring of EvenD8Translations to get the colouring of this triangle's neighborhood
	pub fn new<F>(location: EvenD8Translation, mut colouring: F, requirement_list: &[EvenD8Translation]) -> Triangle
		where F: FnMut(EvenD8Translation) -> Box<dyn SplitInterval> {
			
			
			let mut required = Vec::new();
			for requirement in requirement_list {
				let current = location * (*requirement);
				required.push(colouring(current));
			}
			Triangle{required}
		}

	//(far, close) for a triangle whose "core" lives in its bottom left
	//splitter_list tells each child triangle piece how it came to be
	//CRITICAL
	pub fn split(&self, splitter_list: &[((usize, bool), (usize, bool))]) -> (Triangle, Triangle) {
		let mut next_required_a : Vec<Box<dyn SplitInterval>> = Vec::new();
		let mut next_required_b: Vec<Box<dyn SplitInterval>> = Vec::new();
		
		for ((parent_a, a_type), (parent_b, b_type)) in splitter_list {
			
			next_required_a.push(if *a_type {
				self.required[*parent_a].split().1
			} else {
				self.required[*parent_a].split().0
			});
			
			
			next_required_b.push(if *b_type {
				self.required[*parent_b].split().1
			} else {
				self.required[*parent_b].split().0
			});
		}
		
		(
			Triangle {required: next_required_a},
			Triangle {required: next_required_b},
		)
	}
	
	
	//CRITICAL
	pub fn get_colour(&self, triangle_weights: &[f64]) -> LinearCol {
		//this is just made for the purpose of getting a weighted average. No need to think too hard about what zero transmittance
		//vs zero absorbance is best or what any of it means because weighted averages of meaningful colours are always meaningful
		let mut output_col: LinearCol = LinearCol::new(0.0, 0.0, 0.0, 0.0);
		for (weight, colouring) in zip(triangle_weights, &(self.required)) { //is zip real? It's probably a thing for iterators at least...
			let current_col = colouring.get();
			output_col = output_col + *weight * current_col;
		}
		
		output_col
	}
}
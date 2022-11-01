use super::fractal_specification::FractalSpecification;
use super::dihedral_translation::{EvenD8Translation, IDENTITY};

use std::collections::HashSet;
use std::collections::HashMap;

use nalgebra::{DMatrix, DVector};

pub fn get_relevance_list(fractal: &FractalSpecification) -> Vec<EvenD8Translation> {
	//small:
	let (split_a, split_b) = fractal.splitting_policy();
	let (prod_a, prod_b) = fractal.production_policy();
	//large:
	let unsplit_a = split_a.inv();
	let unsplit_b = split_b.inv();
	
	
	//below is my rambling about how to know what unsplit to use in which case
	//-----------------------------------------------------------------------------------------------------------
	
	//the parity idea below is wrong. Determine lower parity by which type of split it takes to go from upper to lower.
	//a should take false parity to blue, and true parity to orange
	//b should take false parity to orange, and true parity to blue
	
	//how do I know which inverter to use after a production? Note parity? What IS parity in a tiling?
	//pick a triangle, any triangle. Colour it blue. Those touching blue are orange and those touching orange are blue.
	//This may be blue and orange rather than black and white, but that doesn't actually matter(?)
	//a is supposed to take black to blue and white to orange
	//b is supposed to take black to orange and white to blue
	//the FractalSpecification should make sure it has the right relative index order between policies to accomodate this
	
	//still, even with this, it isn't possible to directly know whether to make this "orange" become "white" through un-a or "black" through un-b
	//question: will the wrong choice always take you to odd coordinates?
	//it will if we're trying to find requirements rather than relevances <- no it wont? It's possible for both to lead to the same location.
	//However, it's notable that whenever that happens I didn't check if triangles even work with
	//the versions where inverses of production rules do the same translation
	//if finding relevances... one way is just to invert requirements... wait... did I even want relevance in the first place?
	//No... it just turned up afterwards
	
	//solution: simply know whether something is blue within black, blue within white, orange within black, or orange within white
	//there must therefore be markings for which of black and white prod_a and prod_b produce into
	
	//-----------------------------------------------------------------------------------------------------------
	
	//"find" (or rather assert) that the identity is required
	let mut found_set: HashSet<EvenD8Translation> = HashSet::new();
	let mut relevance_list: Vec<EvenD8Translation> = Vec::new();
	let mut searching: Vec<EvenD8Translation> = Vec::new();
	found_set.insert(IDENTITY);
	relevance_list.push(IDENTITY);
	searching.push(IDENTITY);
	
	while !searching.is_empty() {
		let current_transform = searching.pop().unwrap(); //obviously nonempty because above
		
		//thread::sleep(time::Duration::from_secs(1));
		//println!("\nSearching {:?}", current_transform);
		//println!("unsplit_a {:?}", unsplit_a);
		//println!("unsplit_b {:?}", unsplit_b);
		//println!("product_b {:?}", current_transform * unsplit_b);
		//println!("finalprod {:?}", prod_a * (current_transform * unsplit_b));
		
		//find the two triangles that this triangle produces into
		

		let a_unsplit_a = ( prod_a * (current_transform * unsplit_a) ).expect("Incompatible production");
		let b_unsplit_b = ( prod_b * (current_transform * unsplit_b) ).expect("Incompatible production");
		
		let relevant_a = if fractal.acceptable(a_unsplit_a) {
			a_unsplit_a
		} else {
			( prod_a * (current_transform * unsplit_b) ).expect("Incompatible production")
		};
		
		let relevant_b = if fractal.acceptable(b_unsplit_b) {
			b_unsplit_b
		} else {
			( prod_b * (current_transform * unsplit_a) ).expect("Incompatible production")
		};
		
		//if they're new, make a discovery
		if !found_set.contains(&relevant_a) {
			found_set.insert(relevant_a);
			relevance_list.push(relevant_a);
			searching.push(relevant_a);
		}
		if !found_set.contains(&relevant_b) {
			found_set.insert(relevant_b);
			relevance_list.push(relevant_b);
			searching.push(relevant_b);
		}
	}
	
	relevance_list
}

//this is the one that defines "the kth position" relative to a position.
//current_position * requirement_list[k] is the kth position relative to current_position
pub fn to_requirement_list(relevance_list: &[EvenD8Translation]) -> Vec<EvenD8Translation> {
	relevance_list
		.into_iter()
		.map(|x| x.inv())
		.collect()
}

//entries are: splitter_list[k] = (
//	(which triangle produces into far  child's position k?, how did it split? (false for first way, true for second way)),
//	(which triangle produces into near child's position k?, how did it split? (false for first way, true for second way))
//)
pub fn get_splitter_list(fractal: &FractalSpecification, requirement_list: &[EvenD8Translation]) -> Vec<((usize, bool), (usize, bool))> {
	//make maps for each, then get the final list from that
	let mut  far_hashmap: HashMap<EvenD8Translation, (usize, bool)> = HashMap::new();
	let mut near_hashmap: HashMap<EvenD8Translation, (usize, bool)> = HashMap::new();
	
	for index in 0..requirement_list.len() {
		//add to the far
		let current_far_transform_a = fractal.splitting_policy().0.inv() * requirement_list[index] * fractal.production_policy().0;
		let current_far_transform_b = fractal.splitting_policy().0.inv() * requirement_list[index] * fractal.production_policy().1;
		far_hashmap.insert(current_far_transform_a, (index, false));
		far_hashmap.insert(current_far_transform_b, (index, true));
		//add to the near
		let current_near_transform_a = fractal.splitting_policy().1.inv() * requirement_list[index] * fractal.production_policy().0;
		let current_near_transform_b = fractal.splitting_policy().1.inv() * requirement_list[index] * fractal.production_policy().1;
		near_hashmap.insert(current_near_transform_a, (index, false));
		near_hashmap.insert(current_near_transform_b, (index, true));
	}
	
	//collect to the return vector
	let mut splitter_list: Vec<((usize, bool), (usize, bool))> = Vec::new();
	for transformation in requirement_list {
		//all transformations here should have been seen. For that not to be the case, requirement_list must have been lacking
		let current_entry = (
			far_hashmap .get(transformation).expect("failed to construst splitter list (74218)").to_owned(),
			near_hashmap.get(transformation).expect("failed to construst splitter list (74218)").to_owned()
		);
		splitter_list.push(current_entry);
	}
	
	splitter_list
}


//these "weights" tell you how much of the k_th position fractal you'll find in the current triangle
pub fn get_weights(fractal: &FractalSpecification, relevance_list: &[EvenD8Translation]) -> Vec<f64> {
	//one equation for each triangle (with one dimension of redundancy)
	//a final equation that says the triangle weights sum to 1 (assuming the total fractal area _is_ 1)
	//solve this system of linear equations
	//the following is wrong: weights[k] = (weights[splitter_list[k].0.0] + weights[splitter_list[k].1.0]) / 2
	//splitter_list tells you what triangles split into this one. Not what triangles this one splits into.
	//how to actually make the equations? First initialize the matrix, then I'll put in the right entries. It's the transpose of the thing above.
	//this system of equations is necessarily homogeneous so replace one of them with this:
	//sum weights = 1
	
	//for now, something simple.
	let length = relevance_list.len();
	let (production_a, production_b) = fractal.production_policy();
	let (split_a, split_b) = fractal.splitting_policy();

	//an equation for each triangle, plus an extra one for weights
	let mut coefficients: DMatrix<f64> = DMatrix::zeros(length + 1, length);

	//tells you where in relevancelist to find a transform
	let mut transform_finder: HashMap<EvenD8Translation, usize> = HashMap::new();
	for index in 0..length {
		transform_finder.insert(relevance_list[index], index);
	}

	for index in 0..length {
		let current_transformation = relevance_list[index];
		
		let unsplit_by_a = current_transformation * split_a.inv();
		let unsplit_by_b = current_transformation * split_b.inv();

		let produces_into_a = {
			//these must always unwrap, because we will only ever have even coordinated current_transformations
			let by_a = (production_a * unsplit_by_a).unwrap();
			let by_b = (production_a * unsplit_by_b).unwrap();

			if fractal.acceptable(by_a) && fractal.acceptable(by_b) {
				panic!("help");
			}

			if fractal.acceptable(by_a) {
				by_a
			} else if fractal.acceptable(by_b){
				by_b
			} else {
				panic!("help")
			}
		};

		let produces_into_b = {
			let by_a = (production_b * unsplit_by_a).unwrap();
			let by_b = (production_b * unsplit_by_b).unwrap();

			if fractal.acceptable(by_a) && fractal.acceptable(by_b) {
				panic!("help");
			}

			if fractal.acceptable(by_a) {
				by_a
			} else if fractal.acceptable(by_b){
				by_b
			} else {
				panic!("help")
			}
		};

		//we've found that this triangle produces into image_a with size 1/2,
		//and image_b with size 1/2, so add those weights to the equations
		//(if these images even do anything, that is)
		let &image_a = transform_finder.get(&produces_into_a).expect("what");
		coefficients[(image_a, index)] += 1.0 / 2.0;

		let &image_b = transform_finder.get(&produces_into_b).expect("what");
		coefficients[(image_b, index)] += 1.0 / 2.0;
	}

	for index in 0..length {
		//diagonal (to say the thing above sums to the current triangle for each equation)
		coefficients[(index, index)] -= 1.0;
		//bottom row
		coefficients[(length, index)] = 1.0;
	}

	//coefficients has now been initialized with the appropriate equations
	//all equations are homogeneous except the weights one
	let mut constants: DVector<f64> = DVector::zeros(length + 1);
	constants[length] = 1.0;

	//dump it all into the library function now. It's up to it to deal with the extra equation + dimension of redundancy
	let weights: DVector<f64> = coefficients.svd(true, true).solve(&constants, 0.000001).expect("solve failed?");

	weights
		.into_iter()
		.map(|x| x.to_owned())
		.collect()
}
use std::rc::Rc;

use super::colour_format::LinearCol;
use super::dihedral_translation::{EvenD8Translation, IDENTITY};
use super::dihedral;
use super::interval_colouring::{SplitInterval, ConstantInterval, TreeInterval, FunctionInterval};
use super::square::SquareCut;
use super::triangle::Triangle;

pub enum WorkingMessage {
	Begin,
	End,
}

#[derive(PartialEq, Eq)]
pub enum WorkingState {
	Working,
	Finished,
}

pub fn even_coordinates(transformation: EvenD8Translation) -> bool {
	let (x, y) = transformation.get_translation();
	(x.rem_euclid(2)) == 0 && (y.rem_euclid(2)) == 0
}

pub fn even_coordinates_with_parity(transform: EvenD8Translation) -> bool {
	let (x, y) = transform.get_translation();
	let flip = transform.get_dihedral().flipped();

	(x.rem_euclid(2)) == 0 && (y.rem_euclid(2)) == 0 && !flip
}

pub fn outer_squares_around(transform: EvenD8Translation) -> bool {
	let (x, y) = transform.get_translation();

	let x_halved = if (x.rem_euclid(2)) == 0 {
		x / 2
	} else {return false};
	let y_halved = if (y.rem_euclid(2)) == 0 {
		y / 2
	} else {return false};

	let flip_x = dihedral::EvenD8::new(true, 2);
	let flip_y = dihedral::EvenD8::new(true, 0);

	let mut d8_part = transform.get_dihedral();
	if x_halved.rem_euclid(2) != 0 {
		d8_part = flip_x * d8_part;
	}
	if y_halved.rem_euclid(2) != 0 {
		d8_part = flip_y * d8_part;
	}

	let other_allowable = dihedral::EvenD8::new(true, 3);

	d8_part == dihedral::IDENTITY || d8_part == other_allowable
}

pub fn dragon_acceptable(transform: EvenD8Translation) -> bool {
	let (x, y) = transform.get_translation();
	let d8_part = transform.get_dihedral();

	let x_halved = if (x.rem_euclid(2)) == 0 {
		x / 2
	} else {return false};
	let y_halved = if (y.rem_euclid(2)) == 0 {
		y / 2
	} else {return false};

	let parity_5 = (x_halved + 2 * y_halved).rem_euclid(5);

	match parity_5 {
		0 => d8_part == dihedral::IDENTITY,
		1 => false,
		2 => d8_part == dihedral::EvenD8::new(false, 2),
		3 => d8_part == dihedral::EvenD8::new(false, 3),
		4 => d8_part == dihedral::EvenD8::new(false, 1),
		_ => unreachable!()
	}
}

pub fn simple_colouring (transform: EvenD8Translation) -> Box<dyn SplitInterval> {
	let colour = if transform == IDENTITY {
		LinearCol::new(1.0, 1.0, 1.0, 0.0)
	} else {
		LinearCol::new(0.0, 0.0, 0.0, 0.0)
	};

	let interval: ConstantInterval = colour.into();

	Box::new(interval)
}

pub fn two_colouring (transform: EvenD8Translation) -> Box<dyn SplitInterval> {
	let colour_a = LinearCol::new(1.0, 0.0, 0.0, 0.0);
	let colour_b = LinearCol::new(0.0, 0.0, 1.0, 0.0);
	let background = LinearCol::new(0.0, 0.0, 0.0, 0.0);
	if transform != IDENTITY {
		Box::<ConstantInterval>::new(background.into())
	} else {
		let mid_colour = (colour_a + colour_b) / 2.0;
		let left_interval: ConstantInterval = colour_a.into();
		let right_interval: ConstantInterval = colour_b.into();
		let interval = TreeInterval::new(mid_colour, Rc::new(left_interval), Rc::new(right_interval));
		Box::new(interval)
	}
}

pub fn simple_continuum_colouring (transform: EvenD8Translation) -> Box<dyn SplitInterval> {
	let colour_start = LinearCol::new(1.0, 0.0, 0.0, 0.0);
	let colour_end = LinearCol::new(0.0, 0.0, 1.0, 0.0);
	let background = LinearCol::new(0.0, 0.0, 0.0, 0.0);
	if transform != IDENTITY {
		Box::<ConstantInterval>::new(background.into())
	} else {
		//Since this closure only captures two LinearCols, this closure is pretty cheap to clone.
		//Otherwise, I would have wrapped this in an Rc or something.
		let func = move |x : f64| (1.0 - x) * colour_start + x * colour_end;
		let interval: FunctionInterval<_> = func.into();
		Box::new(interval)
	}
}

pub fn dense_square_config (
	tile_x: isize,
	tile_y: isize,
	requirement_list: &[EvenD8Translation],
	mut triangle_colouring: &mut dyn FnMut(EvenD8Translation) -> Box<dyn SplitInterval>) -> SquareCut {
	
	match (tile_x.rem_euclid(2) == 0, tile_y.rem_euclid(2) == 0) {
		//reasoning: imagine the tiles around the origin
		(true , true ) => SquareCut::Backslash (
				(Triangle::new(
					EvenD8Translation::new(false, 3, (tile_x, -tile_y)),
					&mut triangle_colouring,
					requirement_list
				), false),
				(Triangle::new(
					EvenD8Translation::new(true, 0, (tile_x, -tile_y)),
					&mut triangle_colouring,
					requirement_list
				), true)
			),
		
		(true , false) => SquareCut::Slash (
				(Triangle::new(
					EvenD8Translation::new(true, 1, (tile_x, -tile_y - 1)),
					&mut triangle_colouring,
					requirement_list
				), true),
				(Triangle::new(
					EvenD8Translation::new(false, 0, (tile_x, -tile_y - 1)),
					&mut triangle_colouring,
					requirement_list
				), false)
			),
		(false, true ) => SquareCut::Slash (
				(Triangle::new(
					EvenD8Translation::new(false, 2, (tile_x + 1, -tile_y)),
					&mut triangle_colouring,
					requirement_list
				), false),
				(Triangle::new(
					EvenD8Translation::new(true, 3, (tile_x + 1, -tile_y)),
					&mut triangle_colouring,
					requirement_list
				), true)
			),
		(false, false) => SquareCut::Backslash (
				(Triangle::new(
					EvenD8Translation::new(true, 2, (tile_x + 1, -tile_y - 1)),
					&mut triangle_colouring,
					requirement_list
				), true),
				(Triangle::new(
					EvenD8Translation::new(false, 1, (tile_x + 1, -tile_y - 1)),
					&mut triangle_colouring,
					requirement_list
				), false)
			),
	}
}

pub fn mid_square_config(
	tile_x: isize,
	tile_y: isize,
	requirement_list: &[EvenD8Translation],
	mut triangle_colouring: &mut dyn FnMut(EvenD8Translation) -> Box<dyn SplitInterval>) -> SquareCut {

	//coordinates of... something... in new coordinates
	let new_x = tile_x - tile_y;
	let new_y = -tile_x - tile_y;
	
	if (tile_x + tile_y).rem_euclid(2) == 0 {
		SquareCut::Slash(
			(Triangle::new(
				EvenD8Translation::new(false, 3, (new_x, new_y)),
				&mut triangle_colouring,
				requirement_list
			), false),
			(Triangle::new(
				EvenD8Translation::new(false, 1, (new_x, new_y - 2)),
				&mut triangle_colouring,
				requirement_list
			), false)
		)
	} else {
		SquareCut::Backslash(
			(Triangle::new(
				EvenD8Translation::new(false, 0, (new_x - 1, new_y - 1)),
				&mut triangle_colouring,
				requirement_list
			), false),
			(Triangle::new(
				EvenD8Translation::new(false, 2, (new_x + 1, new_y - 1)),
				&mut triangle_colouring,
				requirement_list
			), false)
		)
	}
}

pub fn large_square_config(
	tile_x: isize,
	tile_y: isize,
	requirement_list: &[EvenD8Translation],
	mut triangle_colouring: &mut dyn FnMut(EvenD8Translation) -> Box<dyn SplitInterval>) -> SquareCut {

	match (tile_x.rem_euclid(2) == 0, tile_y.rem_euclid(2) == 0) {
		//reasoning: imagine the tiles around the origin
		(true , true ) => SquareCut::Backslash (
				(Triangle::new(
					EvenD8Translation::new(true, 3, (tile_x * 2, -tile_y * 2)),
					&mut triangle_colouring,
					requirement_list
				), false),
				(Triangle::new(
					EvenD8Translation::new(false, 0, (tile_x * 2, -tile_y * 2)),
					&mut triangle_colouring,
					requirement_list
				), true)
			),
		
		(true , false) => SquareCut::Slash (
				(Triangle::new(
					EvenD8Translation::new(false, 1, (tile_x * 2, -tile_y * 2)),
					&mut triangle_colouring,
					requirement_list
				), true),
				(Triangle::new(
					EvenD8Translation::new(true, 0, (tile_x * 2, -tile_y * 2)),
					&mut triangle_colouring,
					requirement_list
				), false)
			),
		(false, true ) => SquareCut::Slash (
				(Triangle::new(
					EvenD8Translation::new(true, 2, (tile_x * 2, -tile_y * 2)),
					&mut triangle_colouring,
					requirement_list
				), false),
				(Triangle::new(
					EvenD8Translation::new(false, 3, (tile_x * 2, -tile_y * 2)),
					&mut triangle_colouring,
					requirement_list
				), true)
			),
		(false, false) => SquareCut::Backslash (
				(Triangle::new(
					EvenD8Translation::new(false, 2, (tile_x * 2, -tile_y * 2)),
					&mut triangle_colouring,
					requirement_list
				), true),
				(Triangle::new(
					EvenD8Translation::new(true, 1, (tile_x * 2, -tile_y * 2)),
					&mut triangle_colouring,
					requirement_list
				), false)
			),
	}
}

pub fn dragon_config(
	tile_x: isize,
	tile_y: isize,
	requirement_list: &[EvenD8Translation],
	mut triangle_colouring: &mut dyn FnMut(EvenD8Translation) -> Box<dyn SplitInterval>) -> SquareCut {

	let new_x = 3 * tile_x + tile_y;
	let new_y = tile_x - 3 * tile_y;

	if (tile_x + tile_y).rem_euclid(2) == 0 {
		SquareCut::Backslash(
			(Triangle::new(
				EvenD8Translation::new(false, 3, (new_x, new_y - 2)),
				&mut triangle_colouring,
				requirement_list
			), false),
			(Triangle::new(
				EvenD8Translation::new(false, 1, (new_x + 2, new_y - 2)),
				&mut triangle_colouring,
				requirement_list
			), false)
		)
	} else {
		SquareCut::Slash(
			(Triangle::new(
				EvenD8Translation::new(false, 2, (new_x + 1, new_y - 1)),
				&mut triangle_colouring,
				requirement_list
			), false),
			(Triangle::new(
				EvenD8Translation::new(false, 0, (new_x + 1, new_y - 3)),
				&mut triangle_colouring,
				requirement_list
			), false)
		)
	}
}
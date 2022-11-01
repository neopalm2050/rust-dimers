use super::auxiliary;
//use super::auxiliary::large_square_config;
//use super::auxiliary::mid_square_config;
use super::dihedral_translation::EvenD8Translation;
use super::dihedral_translation::OddD8TranslationSmall;
use super::interval_colouring::SplitInterval;
use super::square::SquareCut;


// how can I hope to get a &'static to a dyn Fn you ask? Well I don't know that well. I can at least leak a Box if it came down to it though.
// I suppose that won't be much of a problem since I won't be making many of these...
pub struct FractalSpecification {
	splitting_policy_tuple: (OddD8TranslationSmall, OddD8TranslationSmall),
	production_policy_tuple: (OddD8TranslationSmall, OddD8TranslationSmall),
	internal_acceptable: &'static (dyn Fn(EvenD8Translation) -> bool),
	get_square_config: &'static (dyn Fn(
		isize,
		isize,
		&[EvenD8Translation],
		&mut dyn FnMut(EvenD8Translation) -> Box<dyn SplitInterval>
	) -> SquareCut),
	splitting_type: (bool, bool),
}

impl FractalSpecification {
	pub fn splitting_policy(&self) -> (OddD8TranslationSmall, OddD8TranslationSmall) {
		self.splitting_policy_tuple.clone()
	}
	
	pub fn production_policy(&self) -> (OddD8TranslationSmall, OddD8TranslationSmall) {
		self.production_policy_tuple.clone()
	}
	
	pub fn acceptable(&self, transformation: EvenD8Translation) -> bool {
		(self.internal_acceptable)(transformation)
	}

	pub fn get_square_config(
		&self,
		x: isize,
		y: isize,
		requirement_list: &[EvenD8Translation],
		triangle_colouring: &mut dyn FnMut(EvenD8Translation) -> Box<dyn SplitInterval>
	) -> SquareCut {
		(self.get_square_config)(x, y, requirement_list, triangle_colouring)
	}

	pub fn get_splitting_type(&self) -> (bool, bool) {
		self.splitting_type
	}
}

pub const CORAL: FractalSpecification = FractalSpecification{
	production_policy_tuple: (
		OddD8TranslationSmall::new(true , 2, ( 1, 0)),
		OddD8TranslationSmall::new(false, 0, (-1, 0))
	),
	
	splitting_policy_tuple: (
		OddD8TranslationSmall::new(true , 1, ( 1, 0)),
		OddD8TranslationSmall::new(false, 1, ( 1, 0))
	),
	
	internal_acceptable: &auxiliary::even_coordinates, //black magic required to find?

	get_square_config: &auxiliary::dense_square_config,

	splitting_type: (false, false),
};

pub const HYDRA: FractalSpecification = FractalSpecification{
	production_policy_tuple: (
		OddD8TranslationSmall::new(true , 0, (-1, 0)),
		OddD8TranslationSmall::new(false, 2, ( 1, 0))
	),
	
	splitting_policy_tuple: (
		OddD8TranslationSmall::new(true , 1, ( 1, 0)),
		OddD8TranslationSmall::new(false, 1, ( 1, 0))
	),
	
	internal_acceptable: &auxiliary::even_coordinates, //black magic required to find?

	get_square_config: &auxiliary::dense_square_config,

	splitting_type: (false, false),
};

pub const DIBOLT: FractalSpecification = FractalSpecification{
	production_policy_tuple: (
		OddD8TranslationSmall::new(true , 2, ( 1, 0)),
		OddD8TranslationSmall::new(true , 3, (-1, 0))
	),
	
	splitting_policy_tuple: (
		OddD8TranslationSmall::new(true , 2, ( 1, 0)),
		OddD8TranslationSmall::new(true , 1, ( 1, 0))
	),
	
	internal_acceptable: &auxiliary::even_coordinates_with_parity, //black magic required to find?

	get_square_config: &auxiliary::mid_square_config,

	splitting_type: (false, true),
};

pub const LEVY: FractalSpecification = FractalSpecification{
	production_policy_tuple: (
		OddD8TranslationSmall::new(false, 3, ( 1, 0)),
		OddD8TranslationSmall::new(false, 0, (-1, 0))
	),
	
	splitting_policy_tuple: (
		OddD8TranslationSmall::new(false, 2, ( 1, 0)),
		OddD8TranslationSmall::new(false, 1, ( 1, 0))
	),
	
	internal_acceptable: &auxiliary::even_coordinates_with_parity, //black magic required to find?

	get_square_config: &auxiliary::mid_square_config,

	splitting_type: (true, false),
};

pub const SCORPION: FractalSpecification = FractalSpecification{
	production_policy_tuple: (
		OddD8TranslationSmall::new(false, 3, ( 1, 0)),
		OddD8TranslationSmall::new(true , 0, (-1, 0))
	),
	
	splitting_policy_tuple: (
		OddD8TranslationSmall::new(true , 2, ( 1, 0)),
		OddD8TranslationSmall::new(false, 2, ( 0, 1))
	),
	
	internal_acceptable: &auxiliary::outer_squares_around, //black magic required to find?

	get_square_config: &auxiliary::large_square_config,

	splitting_type: (false, false),
};

pub const HEIGHWAY: FractalSpecification = FractalSpecification{
	production_policy_tuple: (
		OddD8TranslationSmall::new(false, 1, ( 1, 0)),
		OddD8TranslationSmall::new(false, 0, (-1, 0))
	),
	
	splitting_policy_tuple: (
		OddD8TranslationSmall::new(false, 2, ( 1, 2)),
		OddD8TranslationSmall::new(false, 1, ( 1, 0))
	),
	
	internal_acceptable: &auxiliary::dragon_acceptable, //black magic required to find?

	get_square_config: &auxiliary::dragon_config,

	splitting_type: (true, false),
};
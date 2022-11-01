use std::sync::{Arc, Mutex, mpsc::Sender};
use druid::piet;

use std::cmp::{min, max};

use super::auxiliary::WorkingMessage;
use super::fractal_specification::FractalSpecification;
use super::triangle::Triangle;
use super::interval_colouring::SplitInterval;
use super::colour_format;
use super::dihedral_translation::EvenD8Translation;

pub const SENDING_SIZE: isize = 16;

pub enum SquareCut {
	//the (Triangle, bool) is so that each triangle knows how it's oriented.
	//by default, everything is oriented like Z (for /) or like N (for \)
	Slash     ((Triangle, bool), (Triangle, bool)),
	Backslash ((Triangle, bool), (Triangle, bool)),
}

impl SquareCut {
	//splitting type tells you the orientation of children.
	//false means oriented to the right angle, and true means oriented to the 45 degree angle.
	pub fn split(self, splitter_list: &[((usize, bool), (usize, bool))], splitting_type: (bool, bool)) -> (Self, Self, Self, Self) {
		let (split_a, split_b) = splitting_type;

		//left is oriented to bottom left by default, and everything else works by rotational symmetry
		let (left, top, right, bottom) = match self {
			
			Self::Slash(
				tl,
				br
			) => {
				
				let (left, top) = match tl {
					(tri, false) => {
						let (left, top) = tri.split(splitter_list);
						((left, !split_a), (top, split_b))
					},
					
					(tri, true) => {
						let (top, left) = tri.split(splitter_list);
						((left, !split_b), (top, split_a))
					},
				};

				let (right, bottom) = match br {
					(tri, false) => {
						let (right, bottom) = tri.split(splitter_list);
						((right, !split_a), (bottom, split_b))
					},

					(tri, true) => {
						let (bottom, right) = tri.split(splitter_list);
						((right, !split_b), (bottom, split_a))
					},
				};

				(left, top, right, bottom)
			},

			Self::Backslash(bl, tr) => {

				let (bottom, left) = match bl {
					(tri, false) => {
						let (bottom, left) = tri.split(splitter_list);
						((bottom, !split_a), (left, split_b))
					},

					(tri, true) => {
						let (left, bottom) = tri.split(splitter_list);
						((bottom, !split_b), (left, split_a))
					},
				};

				let (top, right) = match tr {
					(tri, false) => {
						let (top, right) = tri.split(splitter_list);
						((top, !split_a), (right, split_b))
					},

					(tri, true) => {
						let (right, top) = tri.split(splitter_list);
						((top, !split_b), (right, split_a))
					},
				};

				(left, top, right, bottom)
			},
		};

		// The pattern of !s is so that everything is correctly oriented to be recombined
		let (tl_l, bl_l) = match left {
			(tri, false) => {
				let (tl_l, bl_l) = tri.split(splitter_list);
				((tl_l, !split_a), (bl_l, split_b))
			},

			(tri, true ) => {
				let (bl_l, tl_l) = tri.split(splitter_list);
				((tl_l, !split_b), (bl_l, split_a))
			},
		};

		let (tr_t, tl_t) = match top {
			(tri, false) => {
				let (tr_t, tl_t) = tri.split(splitter_list);
				((tr_t, !split_a), (tl_t, split_b))
			},

			(tri, true) => {
				let (tl_t, tr_t) = tri.split(splitter_list);
				((tr_t, !split_b), (tl_t, split_a))
			},
		};

		let (br_r, tr_r) = match right {
			(tri, false) => {
				let (br_r, tr_r) = tri.split(splitter_list);
				((br_r, !split_a), (tr_r, split_b))
			},

			(tri, true) => {
				let (tr_r, br_r) = tri.split(splitter_list);
				((br_r, !split_b), (tr_r, split_a))
			},
		};

		let (bl_b, br_b) = match bottom {
			(tri, false) => {
				let (bl_b, br_b) = tri.split(splitter_list);
				((bl_b, !split_a), (br_b, split_b))
			},

			(tri, true) => {
				let (br_b, bl_b) = tri.split(splitter_list);
				((bl_b, !split_b), (br_b, split_a))
			},
		};


		(
			Self::Backslash(tl_l, tl_t),
			Self::Slash    (tr_t, tr_r),
			Self::Slash    (bl_l, bl_b),
			Self::Backslash(br_b, br_r),
		)
	}
}

//derive all the cool stuff
//(y = down)
//x,y tells you the top-left corner
pub struct UncroppedSquare {
	x: isize,
	y: isize,
	sidelength: isize,
	triangles: SquareCut,
}

pub enum CropOutput {
	Unaffected(UncroppedSquare),
	Cropped(UncroppedSquare, (isize, isize, isize, isize)),
	Empty,
}

impl UncroppedSquare {
	//CRITICAL
	//returns in the order: TL, TR, BL, BR
	pub fn split(self, splitter_list: &[((usize, bool), (usize, bool))], splitting_type: (bool, bool)) -> (UncroppedSquare, UncroppedSquare, UncroppedSquare, UncroppedSquare) {
		
		let half_sidelength = self.sidelength/2;
		
		let (tl_triangles, tr_triangles, bl_triangles, br_triangles) = self.triangles.split(splitter_list, splitting_type);
		
		(
			UncroppedSquare {x: self.x                  , y: self.y                  , sidelength: half_sidelength, triangles: tl_triangles},
			UncroppedSquare {x: self.x + half_sidelength, y: self.y                  , sidelength: half_sidelength, triangles: tr_triangles},
			UncroppedSquare {x: self.x                  , y: self.y + half_sidelength, sidelength: half_sidelength, triangles: bl_triangles},
			UncroppedSquare {x: self.x + half_sidelength, y: self.y + half_sidelength, sidelength: half_sidelength, triangles: br_triangles},
		)
	}
	
	//note: most of the work goes through here and split
	//CRITICAL
	pub fn draw(
		self,
		image_format: piet::ImageFormat,
		canvas: Arc<Mutex<Vec<u8>>>,
		triangle_weights: &[f64],
		canvas_width: usize,
		splitter_list: &[((usize, bool), (usize, bool))],
		splitting_type: (bool, bool),
		sender: Sender<((usize, usize, usize, usize), WorkingMessage)>,
	) {
		if self.sidelength == 1 {
			//find colour, then draw it at x, y
			let square_col: colour_format::LinearCol = match self.triangles {
				//not over, just blended.
				SquareCut::Slash     ((t1, _), (t2, _)) => (t1.get_colour(triangle_weights) + t2.get_colour(triangle_weights)) / 2.0,
				SquareCut::Backslash ((t1, _), (t2, _)) => (t1.get_colour(triangle_weights) + t2.get_colour(triangle_weights)) / 2.0,
			};
			
			match image_format {
				piet::ImageFormat::Grayscale => {
					let brightness: u8 = square_col.grayscale();
					let buffer_pos = (self.y as usize) * canvas_width + (self.x as usize);
					let mut guard = canvas.lock().expect("Canvas lock poisoned");
					guard[buffer_pos] = brightness;
					drop(guard);
				},
				piet::ImageFormat::Rgb => {
					let srgb_col: (u8, u8, u8) = square_col.rgb();
					let buffer_pos = 3 * ((self.y as usize) * canvas_width + (self.x as usize));
					let mut guard = canvas.lock().expect("Canvas lock poisoned");
					guard[buffer_pos  ] = srgb_col.0;
					guard[buffer_pos+1] = srgb_col.1;
					guard[buffer_pos+2] = srgb_col.2;
					drop(guard);
				},
				piet::ImageFormat::RgbaSeparate => {
					let srgba_col: (u8, u8, u8, u8) = square_col.rgba_separate();
					let buffer_pos = 4 * ((self.y as usize)*canvas_width + (self.x as usize));
					let mut guard = canvas.lock().expect("Canvas lock poisoned");
					guard[buffer_pos  ] = srgba_col.0;
					guard[buffer_pos+1] = srgba_col.1;
					guard[buffer_pos+2] = srgba_col.2;
					guard[buffer_pos+3] = srgba_col.3;
					drop(guard);
				},
				piet::ImageFormat::RgbaPremul => {
					let srgba_col: (u8, u8, u8, u8) = square_col.rgba_premul();
					let buffer_pos = 4 * ((self.y as usize) * canvas_width + (self.x as usize));
					let mut guard = canvas.lock().expect("Canvas lock poisoned");
					guard[buffer_pos  ] = srgba_col.0;
					guard[buffer_pos+1] = srgba_col.1;
					guard[buffer_pos+2] = srgba_col.2;
					guard[buffer_pos+3] = srgba_col.3;
					drop(guard);
				},
				_ => panic!("Unsupported colour format"),
			}
			
			return;
		}
		if self.sidelength == 0 {
			panic!("Attempted to draw an empty tile. (35218)");
		}

		let message_bounds = if self.sidelength == SENDING_SIZE {
			let bounds = (
				self.x as usize,
				self.y as usize,
				(self.x + self.sidelength) as usize,
				(self.y + self.sidelength) as usize,
			);

			sender.send((bounds, WorkingMessage::Begin)).expect("sender failed");

			Some(bounds)
		} else {None};
		
		//otherwise,
		let (tl, tr, bl, br) = self.split(splitter_list, splitting_type);
		
		tl.draw(image_format, canvas.clone(), triangle_weights, canvas_width, splitter_list, splitting_type, sender.clone());
		tr.draw(image_format, canvas.clone(), triangle_weights, canvas_width, splitter_list, splitting_type, sender.clone());
		bl.draw(image_format, canvas.clone(), triangle_weights, canvas_width, splitter_list, splitting_type, sender.clone());
		br.draw(image_format, canvas        , triangle_weights, canvas_width, splitter_list, splitting_type, sender.clone()); //I don't need this arc anymore, so this one can have it

		match message_bounds {
			Some (bounds) => {sender.send((bounds, WorkingMessage::End)).expect("sender failed");},
			None => {},
		}
	}
	
	pub fn crop(self, bounds: (isize, isize, isize, isize)) -> CropOutput {
		//println!("{} {}", self.sidelength, self.x);
		if        (bounds.0 <= self.x) && (bounds.1 <= self.y) && (bounds.2 >= self.x + self.sidelength) && (bounds.3 >= self.y + self.sidelength) {
			CropOutput::Unaffected(self)
		} else if (bounds.2 <= self.x) || (bounds.3 <= self.y) || (bounds.0 >= self.x + self.sidelength) || (bounds.1 >= self.y + self.sidelength) {
			CropOutput::Empty
		} else {
			CropOutput::Cropped(self, bounds)
		}
	}
}

impl CropOutput {
	pub fn draw(
		self,
		image_format: piet::ImageFormat,
		canvas: Arc<Mutex<Vec<u8>>>,
		triangle_weights: &[f64],
		canvas_width: usize,
		splitter_list: &[((usize, bool), (usize, bool))],
		splitting_type: (bool, bool),
		sender: Sender<((usize, usize, usize, usize), WorkingMessage)>,
	) {
		match self {
			//the idea is that this is the main case. We want to eventually forget about this cropping.
			CropOutput::Unaffected(uncropped) => uncropped.draw(image_format, canvas, triangle_weights, canvas_width, splitter_list, splitting_type, sender),
			//the only reason this one exists is that we'd get nonsense by drawing off screen otherwise.
			CropOutput::Cropped(base, bounds) => {
				let message_bounds = if base.sidelength == SENDING_SIZE {
					let bounds = (
						max(bounds.0, base.x) as usize,
						max(bounds.1, base.y) as usize,
						min(bounds.2, base.x + base.sidelength) as usize,
						min(bounds.3, base.y + base.sidelength) as usize
					);
					sender.send((bounds, WorkingMessage::Begin)).expect("sender failed");

					Some(bounds)
				} else {None};

				let (tl, tr, bl, br) = base.split(splitter_list, splitting_type);
				tl.crop(bounds).draw(image_format, canvas.clone(), triangle_weights, canvas_width, splitter_list, splitting_type, sender.clone());
				tr.crop(bounds).draw(image_format, canvas.clone(), triangle_weights, canvas_width, splitter_list, splitting_type, sender.clone());
				bl.crop(bounds).draw(image_format, canvas.clone(), triangle_weights, canvas_width, splitter_list, splitting_type, sender.clone());
				br.crop(bounds).draw(image_format, canvas        , triangle_weights, canvas_width, splitter_list, splitting_type, sender.clone());

				match message_bounds {
					Some (bounds) => {sender.send((bounds, WorkingMessage::End)).expect("sender failed");}
					None => {}
				}
			},
			CropOutput::Empty => {},
		}
	}
}


//then I need to generate tiles for a bounding box, then crop them appropriately
//bounds say where in the canvas to draw the thing
//origin is what you expect
//triangle_colouring maps each triangle (represented by a EvenD8Translation) to the appropriate colouring
//scale is 0 for triangles of sidelength 1, 1 for sidelength 2, 8 for sidelength 256, etc.
pub fn draw_into_canvas<F>(
	image_format: piet::ImageFormat,
	canvas: Arc<Mutex<Vec<u8>>>,
	triangle_weights: &[f64],
	requirement_list: &[EvenD8Translation],
	canvas_width: usize,
	bounds: (isize, isize, isize, isize),
	origin: (isize, isize),
	scale: u32,
	fractal: &FractalSpecification,
	mut triangle_colouring: F,
	splitter_list: &[((usize, bool), (usize, bool))],
	sender: Sender<((usize, usize, usize, usize), WorkingMessage)>
	
	) where F : FnMut(EvenD8Translation) -> Box<dyn SplitInterval> {
	
	let sidelength = 1 << scale;
	let shifted_bounds: (isize, isize, isize, isize) = (
		(bounds.0 as isize) - origin.0,
		(bounds.1 as isize) - origin.1,
		(bounds.2 as isize) - origin.0,
		(bounds.3 as isize) - origin.1,
	);

	let splitting_type = fractal.get_splitting_type();
	
	//introducing tile coordinates! A tile coordinate labels points in a sidelength sized grid.
	//note: if perfectly alignes on right and bottom edges, will make useless tiles on the edges.
	//It's not a problem since they'll get cropped out of existence anyway though.
	let  leftmost_tile_x = shifted_bounds.0 >> scale; //this finds how many sidelengths left of the origin you should start the leftmost tiles
	let   highest_tile_y = shifted_bounds.1 >> scale;
	let rightmost_tile_x = shifted_bounds.2 >> scale; //this finds how many sidelengths right of the origin you should start the rightmost tiles
	let    lowest_tile_y = shifted_bounds.3 >> scale;
	
	for current_tile_y in highest_tile_y..=lowest_tile_y {
		for current_tile_x in leftmost_tile_x..=rightmost_tile_x {
			//actually, this'll give different types of tile sepending on the parity of x and y, so account for that.
			let square_config: SquareCut = fractal.get_square_config(current_tile_x, current_tile_y, requirement_list, &mut triangle_colouring);
			
			let uncropped_tile = UncroppedSquare{
				x: (current_tile_x << scale) + origin.0,
				y: (current_tile_y << scale) + origin.1,
				sidelength, //as calculated way at the start
				triangles: square_config,
			};
			
			let cropped_tile = uncropped_tile.crop(bounds);
			
			cropped_tile.draw(image_format, canvas.clone(), triangle_weights, canvas_width, splitter_list, splitting_type, sender.clone());
		}
	}
}
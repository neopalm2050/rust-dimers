use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use druid::piet;

use super::relevance_getter::get_weights;
use super::auxiliary::{self, WorkingMessage};

use super::relevance_getter;
use super::fractal_specification;

use super::square::draw_into_canvas;

use std::{thread, time};

pub fn start(
	width: usize,
	height: usize,
	image_format: piet::ImageFormat,
	canvas: Arc<Mutex<Vec<u8>>>,
	sender: Sender<((usize, usize, usize, usize), WorkingMessage)>) {
	
	thread::sleep(time::Duration::from_secs(0));

	let fractal = &fractal_specification::HYDRA;
	
	let relevance_list = relevance_getter::get_relevance_list(fractal);
	let requirement_list = relevance_getter::to_requirement_list(&relevance_list);
	let splitter_list = relevance_getter::get_splitter_list(fractal, &requirement_list);
	let triangle_weights = get_weights(fractal, &relevance_list);

	draw_into_canvas(
		image_format,
		canvas,
		&triangle_weights,
		&requirement_list,
		width,
		(0, 0, width as isize, height as isize),
		( (width/2) as isize, (height/2) as isize ),
		7,
		fractal,
		&auxiliary::two_colouring,
		&splitter_list,
		sender,
	);
	
	//split space into level=0 blocks (where a block is made of two triangles, and possibly truncated)
		//figure out what cutting each triangle contains
		//recursively fill sub-blocks until you hit the size=1 blocks
		//(this recursive filling is allowed to be async?) <- [figure out how to do that]
		//note: async and multithreading are two different things
		//(if you manage to get this done, there's still the option to move to trimers and make some fractals that are truly your own)
		//(also, make a ui and add "pixel"ation functionality please)
		//warning: if this is async'd, the canvas may be starved for mutex access or something
		//bypass: at a certain level, ask for the tile itself, and only take the mutex once to write the tile there.
		//if the level is too low, there's too much mutex asking, and the canvas will take ages to display
		//if the level is too high, there's too little mutex asking, and the tiles take ages to write themselves to the canvas
		//therefore, in the middle ground, the tiles will only take as much as about a frame to write themselves, leading to perfect drawing rates
		//(however, it will start off as not even async, and mutexes don't even take that long to claim anyway, so the above "issue" could be a myth)
		
		//thinking about it, perhaps the mutex should have been split in the canvas in the first place...
	
	/* for y in 0..height {
		for x in 0..width {
			let buffer_pos = (y*width + x) * image_format.bytes_per_pixel();
			
			let mut guard = canvas.lock().expect("Error: lock poisoned (94743)");
			for byte in 0..image_format.bytes_per_pixel() {
				guard[buffer_pos + byte] = 0xff;
			}
			drop(guard);
		}
		
	} */
	println!("done");
}
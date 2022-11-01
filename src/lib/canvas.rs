use super::auxiliary::{WorkingMessage, WorkingState};
use super::fractal_worker;

use druid::*;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, mpsc};
use std::{thread,time};

const REFRESH_TIME: u64  = 1_000_000 / 10; //microseconds

pub struct FractalBuilder {
	shared_canvas: Arc<Mutex<Vec<u8>>>, // pixelbuffer seer
	own_canvas: Vec<u8>,
	width: usize,
	height: usize,
	format: piet::ImageFormat,
	receiver: Receiver<((usize, usize, usize, usize), WorkingMessage)>,
	working_chunks: HashMap<(usize, usize, usize, usize), WorkingState>,
	_worker: thread::JoinHandle<()>
}

impl FractalBuilder {
	pub fn new(width: usize, height: usize, image_format: piet::ImageFormat) -> Self {
		let size: usize = image_format.bytes_per_pixel() * width * height;
		let own_canvas = vec![0; size];
		let working_chunks = HashMap::new();
		let buffer: Vec<u8> = vec![0; size];
		let shared_canvas = Arc::new(Mutex::new(buffer));
		let canvas_clone = shared_canvas.clone();

		let (sender, receiver) = mpsc::channel::<((usize, usize, usize, usize), WorkingMessage)>();
		
		let worker = thread::spawn(move || {
			fractal_worker::start(width, height, image_format, canvas_clone, sender); //this is where you call the actual builder
		});
		
		FractalBuilder {
			shared_canvas,
			own_canvas,
			width,
			height,
			format: image_format,
			receiver,
			working_chunks,
			_worker: worker
		}
	}
	
	pub fn receive_chunks(&mut self) {
		loop {
			let recv_result = self.receiver.try_recv();

			match recv_result {
				Ok ((bounds, WorkingMessage::Begin)) => {
					self.working_chunks.insert(bounds, WorkingState::Working);
				},

				Ok ((bounds, WorkingMessage::End)) => {
					self.working_chunks.insert(bounds, WorkingState::Finished);
				},

				Err (mpsc::TryRecvError::Empty) => {
					break;
				},

				Err (mpsc::TryRecvError::Disconnected) => {
					break;
				}, //this happens after the builder finishes its work
			}
		}

	}

	pub fn update_interior(&mut self) {

		let mut finished_chunks: Vec<(usize, usize, usize, usize)> = vec![];

		for (bounds, state) in &self.working_chunks {
			if *state == WorkingState::Finished {
				finished_chunks.push(*bounds);
			}

			let colour_depth = self.format.bytes_per_pixel();

			let guard = self.shared_canvas.lock().expect("lock poisoned");
			for y in bounds.1 .. bounds.3 {
				for x in bounds.0 .. bounds.2 {
					let position = ((y * self.width) + x) * colour_depth;
					for byte in 0 .. colour_depth {
						self.own_canvas[position + byte] = guard[position + byte];
					}
				}
			}
			drop(guard);
		}

		for chunk in &finished_chunks {
			self.working_chunks.remove(chunk);
		}
	}
	
	pub fn get_width(&self) -> usize {
		self.width
	}
	
	pub fn get_height(&self) -> usize {
		self.height
	}
	
	pub fn get_format(&self) -> piet::ImageFormat {
		self.format
	}
}


pub struct FractalCanvas {
	builder: FractalBuilder,
	frame_timer: Option<TimerToken>,
}

impl FractalCanvas {
	pub fn new(builder: FractalBuilder) -> FractalCanvas {
		FractalCanvas{builder, frame_timer: None}
	}
}

impl Widget<()> for FractalCanvas {
	fn event(
		&mut self,
		ctx: &mut EventCtx,
		event: &Event,
		_data: &mut (),
		_env: &Env
	) {
		match event {
			Event::WindowConnected => {self.frame_timer = Some(ctx.request_timer(time::Duration::ZERO));},
			Event::Timer(token) => {
				match self.frame_timer {
					Some(timer_token) => if &timer_token == token {
						self.builder.update_interior();
						self.builder.receive_chunks();
						ctx.request_paint();
						ctx.request_timer(time::Duration::from_micros(REFRESH_TIME));
					}
					None => {},
				}
			},
			_ => ()
		}
	}
	
	fn lifecycle(
		&mut self,
		_ctx: &mut LifeCycleCtx,
		_event: &LifeCycle,
		_data: &(),
		_env: &Env
	) {
		()
	}
	
	fn update(
		&mut self,
		_ctx: &mut UpdateCtx,
		_old_data: &(),
		_data: &(),
		_env: &Env
	) {
		()
	}
	
	fn layout(
		&mut self,
		_ctx: &mut LayoutCtx,
		bc: &BoxConstraints,
		_data: &(),
		_env: &Env
	) -> Size {
		bc.constrain(Size::new(
			self.builder.get_width() as f64,
			self.builder.get_height() as f64
		))
	}
	
	fn paint(
		&mut self,
		ctx: &mut PaintCtx,
		_data: &(),
		_env: &Env
	) {
		let image = {
			let width = self.builder.get_width();
			let height = self.builder.get_height();
			let format = self.builder.get_format();
			let buf: &[u8] = &self.builder.own_canvas;
			//slow step? It must be, since it's the only thing I do while the canvas is still locked.
			ctx.make_image(width, height, buf, format).expect("Failed to convert image: 16389")
		};
		
		ctx.draw_image(
			&image,
			Rect::new(
				0.,
				0.,
				self.builder.width as f64,
				self.builder.height as f64
			),
			piet::InterpolationMode::Bilinear
		);
	}
}
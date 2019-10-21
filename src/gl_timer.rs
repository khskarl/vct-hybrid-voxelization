use gl;
use std::collections::HashMap;

type Frame = Vec<f32>;

pub struct GlTimer {
	query_ids: Vec<u32>,
	frames: Vec<Frame>,
	handles: HashMap<String, usize>,
	frame_counter: usize,
	max_frames: usize,
}

impl GlTimer {
	pub fn new(num_queries: usize, max_frames: usize) -> GlTimer {
		let mut query_ids = Vec::<u32>::new();
		query_ids.resize(num_queries, 0);

		unsafe {
			gl::GenQueries(num_queries as i32, query_ids.as_mut_ptr());
		}

		let mut times = Vec::new();
		times.resize(max_frames, 0.0);
		let mut frames = Vec::new();
		frames.resize(num_queries, times);

		GlTimer {
			query_ids,
			frames,
			handles: HashMap::new(),
			frame_counter: 0,
			max_frames,
		}
	}

	pub fn begin_frame(&self) {}

	pub fn end_frame(&mut self) {
		self.frame_counter = (self.frame_counter + 1) % self.max_frames;
	}

	pub fn begin(&mut self, name: &'static str) {
		let id = self.fetch_query_id(name);
		unsafe {
			gl::BeginQuery(gl::TIME_ELAPSED, id);
		}
	}

	pub fn end(&mut self, _name: &'static str) {
		unsafe {
			gl::EndQuery(gl::TIME_ELAPSED);
		}
	}

	fn fetch_query_id(&mut self, name: &'static str) -> u32 {
		let id = if let Some(handle) = self.get_handle(name) {
			let handle = *handle;

			let gl_id = self.query_ids.get_mut(handle).unwrap();
			let mut result = 0;
			unsafe {
				gl::GetQueryObjectuiv(*gl_id as u32, gl::QUERY_RESULT, &mut result);
			}

			let frame = self.frame_counter;
			self.frames[handle][frame] = result as f32 * 0.000_001;

			gl_id
		} else {
			let new_handle = self.handles.len();
			self.handles.insert(name.to_string(), new_handle);

			self
				.query_ids
				.get(new_handle)
				.expect("Not enough queries allocated.")
		};

		*id
	}

	fn get_handle(&self, name: &'static str) -> Option<&usize> {
		self.handles.get(name)
	}

	pub fn save_file(&self, file_name: &str) -> Result<(), ()> {
		use csv::Writer;

		let mut writer = Writer::from_path(file_name).unwrap();
		writer.write_record(&["a", "b", "c"]).unwrap();

		writer.flush().unwrap();

		Ok(())
	}
}

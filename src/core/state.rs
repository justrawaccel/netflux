use std::collections::VecDeque;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ViewMode {
	All,
	DownloadOnly,
	UploadOnly,
}

pub struct AppState {
	pub down_bps: u64,
	pub up_bps: u64,
	pub down_history: VecDeque<u64>,
	pub up_history: VecDeque<u64>,
	pub view_mode: ViewMode,
	pub interface_name: String,
}

impl AppState {
	pub fn new() -> Self {
		Self {
			down_bps: 0,
			up_bps: 0,
			down_history: VecDeque::with_capacity(240),
			up_history: VecDeque::with_capacity(240),
			view_mode: ViewMode::All,
			interface_name: String::new(),
		}
	}

	pub fn update(&mut self, down: u64, up: u64, interface: String) {
		self.down_bps = down;
		self.up_bps = up;
		self.interface_name = interface;

		if self.down_history.len() >= 240 {
			self.down_history.pop_front();
		}
		self.down_history.push_back(down);

		if self.up_history.len() >= 240 {
			self.up_history.pop_front();
		}
		self.up_history.push_back(up);
	}

	pub fn set_view_mode(&mut self, mode: ViewMode) {
		self.view_mode = mode;
	}
}

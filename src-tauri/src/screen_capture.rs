use windows_capture::{
    dxgi_duplication_api::{DxgiDuplicationApi, Error},
    monitor::Monitor,
};

pub struct ScreenCapture {
    duplication: DxgiDuplicationApi,
}

impl ScreenCapture {
    pub fn new(monitor_index: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let monitor = Monitor::from_index(monitor_index)?;
        let duplication = DxgiDuplicationApi::new(monitor)?;

        Ok(Self { duplication })
    }

    pub fn process_frame<F>(&mut self, mut callback: F) -> Result<bool, Box<dyn std::error::Error>>
    where
        F: FnMut(&[u8], u32, u32),
    {
        let width = self.duplication.width();
        let height = self.duplication.height();

        let mut frame = match self.duplication.acquire_next_frame(0) {
            Ok(frame) => frame,

            Err(Error::Timeout) => {
                return Ok(false);
            }

            // Err(Error::AccessLost) => {
            //     self.duplication = self.duplication.recreate()?;
            //     return Ok(false);
            // }
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        let mut buffer = frame.buffer()?;

        callback(buffer.as_raw_buffer(), width, height);

        Ok(true)
    }
}

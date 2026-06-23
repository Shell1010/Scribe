pub struct SfsSession {
    buffer: Vec<u8>,
}

impl SfsSession {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
        }
    }

    pub fn process(&mut self, data: &[u8]) -> Vec<String> {
        self.buffer.extend_from_slice(data);

        let mut extracted_objects = vec![];

        while let Some(null_index) = self.buffer.iter().position(|&b| b == 0x00) {
            let frame_byte = self.buffer.drain(0..null_index).collect::<Vec<u8>>();

            if !self.buffer.is_empty() {
                self.buffer.remove(0);
            }

            if let Ok(raw_value) = String::from_utf8(frame_byte) {
                let trimmed = raw_value.trim().to_string();
                if trimmed.starts_with('{') && trimmed.ends_with('}') {
                    extracted_objects.push(trimmed.to_string());
                }
            }
        }

        extracted_objects
    }
}
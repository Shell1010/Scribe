pub struct SfsSession {
    buffer: Vec<u8>,
}

impl SfsSession {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(4096),
        }
    }

    pub fn process(&mut self, data: &[u8]) -> Vec<String> {
        self.buffer.extend_from_slice(data);

        let mut extracted = Vec::new();
        let mut start = 0;

        while let Some(pos) = self.buffer[start..].iter().position(|&b| b == 0x00) {
            let abs = start + pos;
            if let Ok(raw) = std::str::from_utf8(&self.buffer[start..abs]) {
                let trimmed = raw.trim();
                if trimmed.starts_with('{') && trimmed.ends_with('}') {
                    extracted.push(trimmed.to_string());
                }
            }

            start = abs + 1; 
        }
        
        if start > 0 {
            self.buffer.drain(0..start);
        }

        extracted
    }
}

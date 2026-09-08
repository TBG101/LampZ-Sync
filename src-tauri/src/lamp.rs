use rustuya::sync::Device;
use serde_json::json;
use std::error::Error;

const DP_POWER: u8 = 20;
const DP_MODE: u8 = 21;
const DP_BRIGHTNESS: u8 = 22;
const DP_TEMPERATURE: u8 = 23;
const DP_COLOR: u8 = 24;
const DP_CONTROL: u8 = 28;

pub struct Lamp {
    device: Device,
}

impl Lamp {
    pub fn new(device_id: &str, ip: &str, local_key: &str) -> Result<Self, Box<dyn Error>> {
        let device = Device::new(device_id, local_key);

        device.set_address(ip);
        device.set_nowait(false);
        Ok(Self { device })
    }

    pub fn on(&self) -> Result<(), Box<dyn Error>> {
        self.device.set_value(DP_POWER, true)?;
        Ok(())
    }

    pub fn off(&self) -> Result<(), Box<dyn Error>> {
        self.device.set_value(DP_POWER, false)?;
        Ok(())
    }

    pub fn set_mode(&self, mode: &str) -> Result<(), Box<dyn Error>> {
        match mode {
            "white" | "colour" | "scene" | "music" => {}
            _ => return Err(format!("invalid lamp mode: {mode}").into()),
        }

        self.device.set_value(DP_MODE, mode.to_string())?;

        Ok(())
    }

    pub fn colour_mode(&self) -> Result<(), Box<dyn Error>> {
        self.set_mode("colour")
    }

    pub fn white_mode(&self) -> Result<(), Box<dyn Error>> {
        self.set_mode("white")
    }

    pub fn set_brightness(&self, brightness: u16) -> Result<(), Box<dyn Error>> {
        let brightness = brightness.clamp(10, 1000);

        self.device.set_value(DP_BRIGHTNESS, brightness)?;

        Ok(())
    }

    pub fn set_temperature(&self, temperature: u16) -> Result<(), Box<dyn Error>> {
        let temperature = temperature.min(1000);

        self.device.set_value(DP_TEMPERATURE, temperature)?;

        Ok(())
    }

    // DP 24
    //
    // H: 0..360
    // S: 0..1000
    // V: 0..1000
    //
    // Encoded as:
    //
    // HHHH SSSS VVVV
    //
    // Example:
    // 120, 1000, 1000
    // -> 007803e803e8
    pub fn set_hsv(&self, h: u16, s: u16, v: u16) -> Result<(), Box<dyn Error>> {
        let h = h.min(360);
        let s = s.min(1000);
        let v = v.min(1000);

        let value = format!("{:04x}{:04x}{:04x}", h, s, v);

        self.device.set_value(DP_COLOR, value)?;

        Ok(())
    }

    // DP 28
    //
    // H: 0..360
    // S: 0..255
    // V: 0..255
    pub fn control_hsv(&self, h: u16, s: u8, v: u8, mode: &str) -> Result<(), Box<dyn Error>> {
        let h = h.min(360);

        if mode != "direct" && mode != "gradient" {
            return Err("mode must be 'direct' or 'gradient'".into());
        }

        let brightness = ((v as u32) * 1000 / 255) as u16;

        let value = json!({
            "change_mode": mode,
            "bright": brightness,
            "temperature": 0,
            "h": h,
            "s": s,
            "v": v
        });

        self.device.set_value(DP_CONTROL, value)?;

        Ok(())
    }

    pub fn set_color(&self, h: u16, s: u8, v: u8) -> Result<(), Box<dyn Error>> {
        self.control_hsv(h, s, v, "direct")
    }

    pub fn set_gradient(&self, h: u16, s: u8, v: u8) -> Result<(), Box<dyn Error>> {
        self.control_hsv(h, s, v, "gradient")
    }

    pub fn status(&self) -> Result<Option<String>, Box<dyn Error>> {
        Ok(self.device.status()?)
    }

    pub fn is_connected(&self) -> bool {
        return self.device.is_connected();
    }
}

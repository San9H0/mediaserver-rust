use crate::hubs::unit::HubUnit;

pub trait Session {
    fn run2(&self);

    fn on_video(&self, unit: &HubUnit) {
        // do nothing
    }

    fn on_audio(&self, unit: &HubUnit) {
        // do nothing
    }

    fn read_hub_stream(&self) {
        println!("myrun");
        self.run2();
    }
}


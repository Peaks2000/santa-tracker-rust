//! Port of santa-weather.js's normalized snow/cloud simulation.
use crate::random::Random;
pub struct Flake {
    pub x: f64,
    pub y: f64,
    pub ry: f64,
    pub d: f64,
    pub opacity: f64,
}
pub struct Cloud {
    pub image: usize,
    pub width: f64,
    pub height: f64,
    pub weight: f64,
    pub x: f64,
    pub y: f64,
    pub opacity: f64,
}
pub struct Weather {
    pub flakes: Vec<Flake>,
    pub clouds: Vec<Cloud>,
    random: Random,
    wind: f64,
    current_wind: f64,
    wind_time: f64,
}
impl Weather {
    pub fn new(seed: &str) -> Self {
        let mut random = Random::seeded(seed);
        let wind = 0.25 - random.next() * 0.5;
        let wind_time = random.next() * 15.;
        Self {
            flakes: vec![],
            clouds: vec![],
            random,
            wind,
            current_wind: 0.,
            wind_time,
        }
    }
    fn flake(&mut self, initial: bool) -> Flake {
        let (x, y) = if initial {
            (self.random.next(), self.random.next())
        } else {
            let pos = self.random.next();
            if pos < 1. / 3. {
                (-0.1, self.random.next())
            } else if pos > 2. / 3. {
                (1.1, self.random.next())
            } else {
                (self.random.next(), 0.)
            }
        };
        Flake {
            x,
            y,
            ry: 0.5 + self.random.next(),
            d: 2. + self.random.next() * 2.,
            opacity: 0.6 + self.random.next() * 0.4,
        }
    }
    fn cloud(&mut self, initial: bool, width: f64) -> Cloud {
        let w = 50. + self.random.next() * 100.;
        let h = w * 0.6;
        let offset = (w / width) / 2.;
        let x = if initial {
            self.random.next()
        } else if self.wind > 0. {
            -offset
        } else {
            1. + offset
        };
        Cloud {
            image: (self.random.next() * 3.).floor() as usize,
            width: w,
            height: h,
            weight: 4. - w / 50.,
            x,
            y: self.random.next() * 0.4,
            opacity: 0.2 + self.random.next() * 0.7,
        }
    }
    pub fn step(&mut self, width: f64, height: f64) {
        self.wind_time -= 1. / 60.;
        if self.wind_time <= 0. {
            self.wind = 0.25 - self.random.next() * 0.5;
            self.wind_time = self.random.next() * 15.;
        }
        if self.flakes.is_empty() {
            for _ in 0..40 {
                let f = self.flake(true);
                self.flakes.push(f);
            }
        } else {
            for i in 0..self.flakes.len() {
                let f = &mut self.flakes[i];
                f.x += 0.01 * self.current_wind / f.d;
                f.y += 0.0002 * f.d * f.ry;
                if f.y * height > height || f.x < -0.1 || f.x > 1.1 {
                    self.flakes[i] = self.flake(false);
                }
            }
        }
        if self.clouds.is_empty() {
            for _ in 0..6 {
                let c = self.cloud(true, width);
                self.clouds.push(c);
            }
        } else {
            for i in 0..self.clouds.len() {
                let c = &mut self.clouds[i];
                c.x += 0.001 * self.current_wind * c.weight;
                if self.wind > 0. && c.x * width > width + c.width
                    || self.wind < 0. && c.x * width < -c.width
                {
                    self.clouds[i] = self.cloud(false, width);
                }
            }
        }
        self.current_wind += (self.wind - self.current_wind) / 200.;
    }
}

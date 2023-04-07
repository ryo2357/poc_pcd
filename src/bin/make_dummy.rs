use std::fs::File;
use std::io::Write;

#[derive(Debug, Clone)]
struct Pt {
    x: f32,
    y: f32,
    z: f32,
    color_code: u32,
}

impl Pt {
    fn new(x: f32, y: f32, z: f32) -> Pt {
        let red: u8 = x as u8;
        let green: u8 = y as u8;
        let yellow: u8 = z as u8;
        let color_code: u32 = ((red as u32) << 16) | ((green as u32) << 8) | (yellow as u32);
        Pt {
            x,
            y,
            z,
            color_code,
        }
    }
}

struct PtVecBuilder {
    pt: Pt,
    s: f32,
    b: f32,
    r: f32,
    dt: f32,
}

impl PtVecBuilder {
    fn new(pt: Pt, s: f32, b: f32, r: f32, dt: f32) -> Self {
        Self { pt, s, b, r, dt }
    }
    fn next(&mut self, pt: Pt) -> Pt {
        self.pt = pt;
        let x = self.pt.x - self.dt * self.s * (self.pt.x - self.pt.y);
        let y = self.pt.y + self.dt * (-self.pt.x * self.pt.z + self.r * self.pt.x - self.pt.y);
        let z = self.pt.z + self.dt * (self.pt.x * self.pt.y - self.b * self.pt.z);
        Pt::new(x, y, z)
    }
    fn build(&mut self, num: usize) -> Vec<Pt> {
        let mut vec = Vec::new();
        let mut pt = self.pt.clone();
        for _ in 0..num {
            vec.push(pt.clone());
            pt = self.next(pt);
        }
        vec
    }
}

fn main() {
    //***********************************************************
    // 適当な点群リストを作成

    let pt = Pt::new(7.0, 23.0, 35.0);
    let s = 10.0;
    let b = 1.8;
    let r = 20.0;
    let dt = 0.0001;

    let mut builder = PtVecBuilder::new(pt, s, b, r, dt);
    let pts = builder.build(500000);
    //***********************************************************
    // pcdファイル（バイナリ形式）に出力
    let mut fp = File::create("test.pcd").expect("Could not create file");
    // ヘッダ
    write!(
        fp,
        "# .PCD v.7 - Point Cloud Data file format\nVERSION .7\nFIELDS x y z rgb\nSIZE 4 4 4 4\nTYPE F F F U\nCOUNT 1 1 1 1\nWIDTH {}\nHEIGHT 1\nVIEWPOINT 0 0 0 1 0 0 0\nPOINTS {}\nDATA binary\n",
        pts.len(),
        pts.len()
    )
    .expect("Could not write to file");

    // 中身をバイナリで書き込み
    for pt in &pts {
        fp.write_all(&pt.x.to_le_bytes())
            .expect("Could not write to file");
        fp.write_all(&pt.y.to_le_bytes())
            .expect("Could not write to file");
        fp.write_all(&pt.z.to_le_bytes())
            .expect("Could not write to file");
        fp.write_all(&pt.color_code.to_le_bytes())
            .expect("Could not write to file");
    }
}

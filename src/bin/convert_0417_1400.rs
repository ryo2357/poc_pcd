use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
// 計測失敗点は省かないとビューアーで醜い

fn main() {
    let input_path = "input/data_2023-03-17_084632.hex";
    let output_path = "output/test_0407_1400.pcd";
    let profile_num: usize = 3200;
    let profile_size: usize = 3200;
    let mut converter =
        LJXRowDataFileConverter::new(input_path, output_path, profile_num, profile_size);
    match converter.execute() {
        Ok(_) => {
            println!("変換成功");
            println!("point_num:{:?}", converter.point_num);
        }
        Err(err) => {
            println!("変換失敗");
            panic!("{:?}", err);
        }
    };
}

struct LJXRowDataFileConverter {
    input_path: String,
    output_path: String,
    profile_num: usize,
    profile_size: usize,
    point_num: usize,
}
impl LJXRowDataFileConverter {
    fn new(input_path: &str, output_path: &str, profile_num: usize, profile_size: usize) -> Self {
        Self {
            input_path: input_path.to_string(),
            output_path: output_path.to_string(),
            profile_num: profile_num,
            profile_size: profile_size,
            point_num: 0,
        }
    }
    fn execute(&mut self) -> anyhow::Result<()> {
        let mut reader = LJXRowDataStreamReader::new(&self.input_path)?;
        let mut writer = PcdStreamWriter::new(&self.output_path)?;

        let point_num = self.profile_num * self.profile_size;
        writer.write_header(point_num)?;

        // x_pitch, y_pitchともに2.5μ　⇒　250
        let mut pcd_profile_builder = PcdProfilePointsBuilder::new(250, 250);

        for _i in 0..self.profile_num {
            let profile = reader.read_profile()?;
            let pcd_profile = pcd_profile_builder.make_points(profile);
            writer.write_points(pcd_profile)?;
            // writer.write_points_as_csv(pcd_profile)?;
        }

        Ok(())
    }
}

struct LJXRowDataStreamReader {
    reader: BufReader<File>,
    // TODO:いらないような気がする
    profile_byte_size: usize,
}
impl LJXRowDataStreamReader {
    fn new(file_path: &str) -> anyhow::Result<Self> {
        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);
        let profile_byte_size = LJXProfile::get_profile_byte_size();
        Ok(Self {
            reader: reader,
            profile_byte_size: profile_byte_size,
        })
    }
    fn read_profile(&mut self) -> anyhow::Result<LJXProfile> {
        let mut buffer = LJXProfile::make_read_buf();
        self.reader.read(&mut buffer)?;
        let profile = LJXProfile::new(buffer)?;
        Ok(profile)
    }
}

struct LJXProfile {
    inner: [i32; 3200],
}
impl LJXProfile {
    fn make_read_buf() -> [u8; (3200 + 4) * 4] {
        [0; (3200 + 4) * 4]
    }
    fn get_profile_byte_size() -> usize {
        (3200 + 4) * 4
    }
    fn new(buf: [u8; (3200 + 4) * 4]) -> anyhow::Result<Self> {
        //バッファからi32への変換処理
        let iter = buf.chunks(4).skip(4);
        let mut array: [i32; 3200] = [0; 3200];
        for (i, elem) in iter.enumerate() {
            // この変換だとバイト逆順で変換してる気がする
            // array[i] = i32::from_be_bytes(elem.try_into()?)
            // 補正処理
            array[i] = i32::from_le_bytes(elem.try_into()?);
            // LJX8020,計測範囲1/4の補正
            // 単位は100nmとする
            // もともとこの条件で記録されていた
        }

        Ok(Self { inner: array })
    }
}
struct PcdPoint {
    x: i32,
    y: i32,
    z: i32,
}

enum ProfilePoint {
    Success(PcdPoint),
    Failure,
}

struct PcdProfilePoints {
    inner: Vec<ProfilePoint>,
}

impl PcdProfilePoints {
    fn new() -> Self {
        Self {
            inner: Vec::<ProfilePoint>::new(),
        }
    }
}

struct PcdProfilePointsBuilder {
    next_y: i32,
    y_pitch: i32,
    x_pitch: i32,
}
impl PcdProfilePointsBuilder {
    fn new(y_pitch: i32, x_pitch: i32) -> Self {
        Self {
            next_y: 0,
            y_pitch: y_pitch,
            x_pitch: x_pitch,
        }
    }
    fn make_points(&mut self, profile: LJXProfile) -> PcdProfilePoints {
        let mut vec = PcdProfilePoints::new();

        let mut x = 0;
        for point in profile.inner.iter() {
            let pcd_point = match *point {
                -2147483648 => ProfilePoint::Failure,
                -2147483647 => ProfilePoint::Failure,
                -2147483646 => ProfilePoint::Failure,
                -2147483645 => ProfilePoint::Failure,
                _ => ProfilePoint::Success(PcdPoint {
                    x: x,
                    y: self.next_y,
                    z: *point,
                }),
            };
            // let pcd_point = PcdPoint {
            //     x: x,
            //     y: self.next_y,
            //     z: *point,
            // };
            vec.inner.push(pcd_point);
            x += self.x_pitch;
        }
        self.next_y += self.y_pitch;

        vec
    }
}

struct PcdStreamWriter {
    writer: BufWriter<File>,
    count: usize,
    profile_point_num: usize,
}

impl PcdStreamWriter {
    fn new(file_path: &str) -> anyhow::Result<Self> {
        let file = File::create(&file_path)?;
        let writer = BufWriter::new(file);
        // TODO:値のハードコード
        let profile_point_num = 3200;
        Ok(Self {
            writer: writer,
            count: 0,
            profile_point_num: profile_point_num,
        })
    }

    fn write_points(&mut self, points: PcdProfilePoints) -> anyhow::Result<()> {
        for pt in &points.inner {
            match pt {
                ProfilePoint::Failure => {
                    // headerを後から調整できないのでとりあえず邪魔にならない点にプロット
                    let dummy_num: i32 = -1000;
                    self.writer.write_all(&dummy_num.to_le_bytes())?;
                    self.writer.write_all(&dummy_num.to_le_bytes())?;
                    self.writer.write_all(&dummy_num.to_le_bytes())?;
                }
                ProfilePoint::Success(point) => {
                    self.writer.write_all(&point.x.to_le_bytes())?;
                    self.writer.write_all(&point.y.to_le_bytes())?;
                    self.writer.write_all(&point.z.to_le_bytes())?;
                }
            }
            // self.writer.write_all(&pt.x.to_le_bytes())?;
            // self.writer.write_all(&pt.y.to_le_bytes())?;
            // self.writer.write_all(&pt.z.to_le_bytes())?;
        }
        self.count += self.profile_point_num;
        Ok(())
    }

    fn write_header(&mut self, point_num: usize) -> anyhow::Result<()> {
        // write_profileがすべて終わった後に実行したいが
        // 先頭に追記するためにずらすのはそこそこ重たい処理
        // ⇒　最初に追記する
        let header:String = format!(
            "# .PCD v.7 - Point Cloud Data file format\nVERSION .7\nFIELDS x y z\nSIZE 4 4 4\nTYPE I I I\nCOUNT 1 1 1\nWIDTH {}\nHEIGHT 1\nVIEWPOINT 0 0 0 1 0 0 0\nPOINTS {}\nDATA binary\n",
            point_num,
            point_num
        );
        self.writer.write_all(header.as_bytes())?;
        Ok(())
    }

    fn write_points_as_csv(&mut self, points: PcdProfilePoints) -> anyhow::Result<()> {
        for pt in &points.inner {
            match pt {
                ProfilePoint::Failure => continue,
                ProfilePoint::Success(point) => {
                    let point_string: String = format!("{},{},{}\n", point.x, point.y, point.z,);
                    self.writer.write_all(point_string.as_bytes())?;
                }
            }
        }
        self.count += self.profile_point_num;
        Ok(())
    }
}

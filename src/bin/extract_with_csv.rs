// Open3dならロードできるがPymeshlabではロードできな
// propertyをdouble(f64)

use dotenv::dotenv;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::process::Command;

fn main() {
    dotenv().ok();
    let input_path = env::var("INPUT_PATH").unwrap();
    let output_path = env::var("OUTPUT_PATH").unwrap();

    // 168,000プロファイル　70,000
    // 537,600,000点から160,000点を流出して描画⇒0.02％
    let profile_start_num = 100_000;
    let profile_take_num = 400;

    let profile_range_start_num = 1700;
    let profile_range_take_num = 400;

    let mut converter = LJXFileConverterBuilder::new()
        .set_reader(input_path.to_owned())
        .set_profile_range(profile_start_num, profile_take_num)
        .set_parser(RowDataToProfile::new(profile_range_start_num, profile_range_take_num).unwrap())
        .set_converter(ProfileToPcd::new(250.0, 250.0))
        .set_writer(output_path.to_owned())
        .build()
        .unwrap();
    match converter.execute() {
        Ok(_) => {
            println!("変換成功");
        }
        Err(err) => {
            println!("変換失敗");
            panic!("{:?}", err);
        }
    };

    // ファイル情報の作成
    let info_file_path = String::new() + &output_path.replace(".ply", "_info.txt");
    println!("{:?}", info_file_path);
    // 出力完了しているのディレクトエイはあるはず
    // let folder_file = std::path::Path::new(&info_file_path).parent().unwrap();
    // fs::create_dir_all(folder_file).unwrap();

    let mut file = File::create(info_file_path).unwrap();
    writeln!(file, "input_file:{:?}", input_path).unwrap();
    writeln!(file, "profile_start_num:{:?}", profile_start_num).unwrap();
    writeln!(file, "profile_take_num:{:?}", profile_take_num).unwrap();
    writeln!(
        file,
        "profile_range_start_num:{:?}",
        profile_range_start_num
    )
    .unwrap();
    writeln!(file, "profile_range_take_num:{:?}", profile_range_take_num).unwrap();

    // メッシュの作成
    make_mash_file(output_path.to_owned()).unwrap();

    // csvの作成
    let csv_file_path = String::new() + &output_path.replace(".ply", ".csv");

    let mut converter = LJXFileConverterBuilder::new()
        .set_reader(input_path.to_owned())
        .set_profile_range(profile_start_num, profile_take_num)
        .set_parser(RowDataToProfile::new(profile_range_start_num, profile_range_take_num).unwrap())
        .set_converter(ProfileToPcd::new(250.0, 250.0))
        .set_writer(csv_file_path)
        .build()
        .unwrap();
    match converter.execute_csv() {
        Ok(_) => {
            println!("変換成功");
        }
        Err(err) => {
            println!("変換失敗");
            panic!("{:?}", err);
        }
    };
}
fn make_mash_file(input_path: String) -> anyhow::Result<()> {
    let python_path = env::var("PYTHON_PATH").unwrap();
    let script_path = env::var("SCRIPT_PATH").unwrap();
    let temp_file_path = String::new() + &input_path.replace(".ply", "_with_mesh.ply");
    let jpg_file_path = String::new() + &input_path.replace(".ply", ".jpg");
    println!("{:?}", temp_file_path);
    println!("{:?}", jpg_file_path);

    let output = Command::new(python_path)
        .arg(script_path)
        .arg(input_path)
        .arg(temp_file_path)
        .arg(jpg_file_path)
        .output();

    match output {
        Ok(s) => println!("成功:{:?}", String::from_utf8_lossy(&s.stdout)),
        Err(err) => println!("{:?}", err),
    }

    println!("完了");
    Ok(())
}

#[derive(Default)]
struct LJXFileConverterBuilder {
    input_file_path: Option<String>,
    parser: Option<RowDataToProfile>,
    profile_start: Option<usize>,
    profile_take_num: Option<usize>,
    converter: Option<ProfileToPcd>,
    output_file_path: Option<String>,
}
impl LJXFileConverterBuilder {
    fn new() -> Self {
        LJXFileConverterBuilder::default()
    }
    fn set_writer(mut self, output_path: String) -> Self {
        self.output_file_path = Some(output_path);
        self
    }
    fn set_reader(mut self, input_path: String) -> Self {
        self.input_file_path = Some(input_path);
        self
    }
    fn set_parser(mut self, parser: RowDataToProfile) -> Self {
        self.parser = Some(parser);
        self
    }
    fn set_converter(mut self, converter: ProfileToPcd) -> Self {
        self.converter = Some(converter);
        self
    }
    fn set_profile_range(mut self, profile_start: usize, profile_take_num: usize) -> Self {
        self.profile_start = Some(profile_start);
        self.profile_take_num = Some(profile_take_num);
        self
    }
    fn build(self) -> anyhow::Result<LJXDataFileConverter> {
        let reader =
            LJXRowDataStreamReader::new(&self.input_file_path.unwrap(), self.parser.unwrap())?;
        let writer = PcdStreamWriter::new(&self.output_file_path.unwrap())?;
        Ok(LJXDataFileConverter {
            writer: writer,
            reader: reader,
            converter: self.converter.unwrap(),
            profile_start: self.profile_start.unwrap(),
            profile_take_num: self.profile_take_num.unwrap(),
        })
    }
}
struct LJXDataFileConverter {
    writer: PcdStreamWriter,
    reader: LJXRowDataStreamReader,
    converter: ProfileToPcd,
    profile_start: usize,
    profile_take_num: usize,
    // プロファイル数の管理
}
impl LJXDataFileConverter {
    fn execute(&mut self) -> anyhow::Result<()> {
        // 先頭に追記の処理が難しいので後で手動で変更する
        self.writer.write_header(55)?;

        for _i in 0..self.profile_start {
            self.reader.skip_read()?;
        }

        for _i in 0..self.profile_take_num {
            let profile = self.reader.read_profile()?;
            let pcd_profile = self.converter.make_points(profile);

            self.writer.write_points(pcd_profile)?;
        }
        println!("ポイント点数{:?}", self.writer.get_point_count());

        self.writer.fix_header()?;

        Ok(())
    }
    fn execute_csv(&mut self) -> anyhow::Result<()> {
        for _i in 0..self.profile_start {
            self.reader.skip_read()?;
        }

        for _i in 0..self.profile_take_num {
            let profile = self.reader.read_profile()?;
            let pcd_profile = self.converter.make_points(profile);
            self.writer.write_points_as_csv(pcd_profile)?;
        }
        println!("ポイント点数{:?}", self.writer.get_point_count());

        Ok(())
    }
}

struct LJXRowDataStreamReader {
    reader: BufReader<File>,
    parser: RowDataToProfile,
}
impl LJXRowDataStreamReader {
    fn new(file_path: &str, parser: RowDataToProfile) -> anyhow::Result<Self> {
        let file = File::open(&file_path)?;
        let reader = BufReader::new(file);
        Ok(Self {
            reader: reader,
            parser: parser,
        })
    }
    fn read_profile(&mut self) -> anyhow::Result<LJXProfile> {
        let mut buffer = self.parser.make_read_buf();
        let _len = self.reader.read(&mut buffer)?;
        // len == 0　でエラーハンドリングするべき?
        let profile = self.parser.parse(buffer)?;
        Ok(profile)
    }

    fn skip_read(&mut self) -> anyhow::Result<()> {
        let mut buffer = self.parser.make_read_buf();
        let _len = self.reader.read(&mut buffer)?;
        Ok(())
    }
}

struct LJXProfile {
    inner: Vec<i32>,
}

// ハードコードされているほうが早い？
struct RowDataToProfile {
    start: usize,
    take_num: usize,
}

impl RowDataToProfile {
    fn new(start: usize, take_num: usize) -> anyhow::Result<Self> {
        // TODO:条件式が本当にあっているか確認が必要
        if start + take_num > 3200 {
            anyhow::bail!("RowDataToProfileの入力値が不正")
        }
        Ok(Self {
            start: start,
            take_num: take_num,
        })
    }
    // 輝度データ無し
    // fn make_read_buf(&self) -> [u8; (3200 + 4) * 4] {
    //     [0; (3200 + 4) * 4]
    // }
    // fn parse(&mut self, buf: [u8; (3200 + 4) * 4]) -> anyhow::Result<LJXProfile> {
    //     //バッファからi32への変換処理
    //     let iter = buf.chunks(4).skip(4).skip(self.start).take(self.take_num);
    //     let mut vec = Vec::<i32>::new();
    //     for buf in iter {
    //         vec.push(i32::from_le_bytes(buf.try_into()?));
    //         // 単位は100nmとする 0.1μm
    //     }

    //     Ok(LJXProfile { inner: vec })
    // }
    // 輝度データ有り
    fn make_read_buf(&self) -> [u8; (3200 + 3200 + 4) * 4] {
        [0; (3200 + 3200 + 4) * 4]
    }
    fn parse(&mut self, buf: [u8; (3200 + 3200 + 4) * 4]) -> anyhow::Result<LJXProfile> {
        //バッファからi32への変換処理
        let iter = buf.chunks(4).skip(4).skip(self.start).take(self.take_num);
        let mut vec = Vec::<i32>::new();
        for (i, buf) in iter.enumerate() {
            if i == 3200 {
                break;
            }
            vec.push(i32::from_le_bytes(buf.try_into()?));
            // 単位は100nmとする 0.1μm
        }

        Ok(LJXProfile { inner: vec })
    }
}

struct ProfileToPcd {
    next_y: f64,
    y_pitch: f64,
    x_pitch: f64,
}
impl ProfileToPcd {
    fn new(y_pitch: f64, x_pitch: f64) -> Self {
        Self {
            next_y: 0.0,
            y_pitch: y_pitch,
            x_pitch: x_pitch,
        }
    }
    fn make_points(&mut self, profile: LJXProfile) -> PcdProfilePoints {
        let mut vec = PcdProfilePoints::new();
        let mut x = 0.0;
        for point in profile.inner.iter() {
            let pcd_point = match *point {
                // 仕様での出力値
                // -2147483648 => ProfilePoint::Failure,
                // -2147483647 => ProfilePoint::Failure,
                // -2147483646 => ProfilePoint::Failure,
                // -2147483645 => ProfilePoint::Failure,

                // 計測範囲外なので出現しないはずの値
                // 12_398_995 周辺の値が発生　⇒　123mm
                // 計測範囲は1.1mm ⇒　-550～550μ　⇒-55000～55000
                i32::MIN..=-55001 => ProfilePoint::Failure,
                55000..=i32::MAX => ProfilePoint::Failure,
                _ => ProfilePoint::Success(PcdPoint {
                    x: x,
                    y: self.next_y,
                    z: f64::from(*point),
                }),
            };
            vec.inner.push(pcd_point);
            x += self.x_pitch;
        }
        self.next_y += self.y_pitch;

        vec
    }
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

struct PcdPoint {
    x: f64,
    y: f64,
    z: f64,
}
impl PcdPoint {
    fn get_point_binary(&self) -> [u8; 24] {
        let mut buf = [0; 24];
        // let x: [u8; 4] = self.x.to_le_bytes();
        // let y: [u8; 4] = self.y.to_le_bytes();
        // let z: [u8; 4] = self.z.to_le_bytes();
        buf[0..8].copy_from_slice(&self.x.to_le_bytes());
        buf[8..16].copy_from_slice(&self.y.to_le_bytes());
        buf[16..24].copy_from_slice(&self.z.to_le_bytes());
        buf
    }
}

enum ProfilePoint {
    Success(PcdPoint),
    Failure,
}

struct PcdStreamWriter {
    writer: BufWriter<File>,
    point_count: usize,
    // TODO:先頭への追記処理に使う
    input_file_path: String,
}

impl PcdStreamWriter {
    fn new(file_path: &str) -> anyhow::Result<Self> {
        let folder_file = std::path::Path::new(&file_path).parent().unwrap();
        fs::create_dir_all(folder_file)?;
        let file = File::create(&file_path)?;
        let writer = BufWriter::new(file);
        Ok(Self {
            writer: writer,
            point_count: 0,
            input_file_path: file_path.to_string(),
        })
    }

    fn get_point_count(&self) -> usize {
        self.point_count
    }

    fn write_points(&mut self, points: PcdProfilePoints) -> anyhow::Result<()> {
        for pt in &points.inner {
            match pt {
                ProfilePoint::Failure => {
                    continue;
                }
                ProfilePoint::Success(point) => {
                    self.writer.write_all(&point.get_point_binary())?;
                    self.point_count += 1;
                }
            }
        }
        Ok(())
    }

    fn write_header(&mut self, point_num: usize) -> anyhow::Result<()> {
        let header = make_header(point_num);
        self.writer.write_all(header.as_bytes())?;
        Ok(())
    }

    fn fix_header(&mut self) -> anyhow::Result<()> {
        let point_num = self.get_point_count();
        let header = make_header(point_num);

        self.writer.seek(SeekFrom::Start(0))?;
        self.writer.write_all(header.as_bytes())?;
        self.writer.flush()?;
        self.writer.seek(SeekFrom::End(0))?;
        Ok(())
    }

    // csv出力用の追加メソッド
    fn write_points_as_csv(&mut self, points: PcdProfilePoints) -> anyhow::Result<()> {
        for pt in &points.inner {
            match pt {
                ProfilePoint::Failure => {
                    continue;
                }
                ProfilePoint::Success(point) => {
                    let point_string: String = format!("{},{},{}\n", point.x, point.y, point.z,);
                    self.writer.write_all(point_string.as_bytes())?;
                    self.point_count += 1;
                }
            }
        }
        Ok(())
    }
}

fn make_header(point_num: usize) -> String {
    let point_digits_size: usize = 20 - format!("{:}", point_num).to_string().len();
    let adjust_comment = &"xxxxxxxxxxxxxxxxxxxx"[0..point_digits_size];
    let header:String = format!(
        "ply\nformat binary_little_endian 1.0\ncomment adjust str {}\nelement vertex {}\nproperty double x\nproperty double y\nproperty double z\nend_header\n",
        adjust_comment,
        point_num
    );
    header
}

pub mod parser;

#[cfg(test)]
mod test {
    use std::io::Read;

    use anyhow::Result;

    use crate::parser::{output_json, parse_flash_animation};

    #[test]
    fn test() -> Result<()> {
        // 模拟读取测试文件
        let file_path = "C:\\Users\\linlin\\Desktop\\skin_test.swf";
        let file = std::fs::File::open(file_path).expect("Failed to open test file");
        let mut reader = std::io::BufReader::new(file);
        let mut data = Vec::new();
        reader
            .read_to_end(&mut data)
            .expect("Failed to read test file");
        // 调用解析函数
        let animations = parse_flash_animation(data).expect("Failed to parse SWF tag");
        output_json(&animations, true, "test", "")?;
        Ok(())
    }
}

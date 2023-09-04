use bevy::prelude::Vec2;

const DELIMITER: &str = ":";

pub fn encode(data: Vec<Vec2>) -> String {
    let mut data = data;
    data.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

    let mut content = String::new();

    for p in data {
        content.push_str(format!("{:.4}{}{:.4}\n", p.x, DELIMITER, p.y).as_str());
    }

    content
}

pub fn decode(data: String) -> Result<Vec<Vec2>, String> {
    let mut result = vec![];

    let mut ind = 1;
    for line in data.lines() {
        let p = line.split_once(DELIMITER);
        if p.is_none() {
            return Err(format!("line {}: unexpected data {}", ind, line));
        }

        let p = p.unwrap();
        let x = match p.0.parse::<f32>() {
            Ok(val) => val,
            Err(err) => {
                return Err(format!(
                    "line {}: x is not float32: {} at {}",
                    ind, err, line
                ))
            }
        };
        let y = match p.1.parse::<f32>() {
            Ok(val) => val,
            Err(err) => {
                return Err(format!(
                    "line {}: y is not float32: {} at {}",
                    ind, err, line
                ))
            }
        };

        result.push(Vec2::new(x, y));
        ind += 1;
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn encode_test() {
        let data = vec![
            Vec2::new(0.543, 0.7432),
            Vec2::new(0.1, 0.99),
            Vec2::new(0.45, 0.88),
            Vec2::new(0.0, 0.05),
            Vec2::new(1.0, 0.5345),
        ];

        assert_eq!(
            encode(data),
            String::from(
                "0.0000:0.0500\n0.1000:0.9900\n0.4500:0.8800\n0.5430:0.7432\n1.0000:0.5345\n"
            )
        );
    }

    #[test]
    pub fn decode_test() {
        let data = vec![
            Vec2::new(0.0, 0.05),
            Vec2::new(0.1, 0.99),
            Vec2::new(0.45, 0.88),
            Vec2::new(0.543, 0.7432),
            Vec2::new(1.0, 0.5345),
        ];

        let decoded = decode(String::from(
            "0.0000:0.0500\n0.1000:0.9900\n0.4500:0.8800\n0.5430:0.7432\n1.0000:0.5345\n",
        ));
        assert!(decoded.is_ok());
        let decoded = decoded.unwrap();

        assert_eq!(decoded, data);
    }

    #[test]
    pub fn decode_err_test() {
        let decoded = decode(String::from("0.0000:0.0500\n0.5:oups\n"));
        assert!(decoded.is_err());

        assert_eq!(
            decoded.unwrap_err(),
            "line 2: y is not float32: invalid float literal at 0.5:oups"
        );
    }
}

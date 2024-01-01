/// Represents the Calver version scheme (calver.org).
///
/// Currently only the "Ubuntu" version of the scheme is supported (YY.0M[.Micro]).
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Calver {
    /// Short year - 6, 16, 106
    short_year: String,
    /// Zero-padded month - 01, 02 ... 11, 12.
    zero_padded_month: String,
    /// The third and usually final number in the version. Sometimes referred to as the "patch" segment.
    micro: Option<String>,
}

impl Calver {
    /// Try creating a Calver object from an Ubuntu-like version string.
    ///
    /// Note that padding issues will be fixed during parsing. For instance "024.012.2",
    /// which is invalid, will be turned into the valid version string "24.12.2".
    pub fn try_from_ubuntu(version: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return Err(format!(
                "version string must contain 2 ou 3 parts (YY.0M[.Micro]), got {:?}",
                parts
            ));
        }

        Ok(Self {
            short_year: Self::short_year_from_str(parts[0])?,
            zero_padded_month: Self::zero_padded_month_from_str(parts[1])?,
            micro: Self::micro_from_str(parts.get(2).copied())?,
        })
    }

    /// Render a Calver object into a valid version string.
    pub fn to_ubuntu(&self) -> String {
        let mut version = format!("{}.{}", self.short_year, self.zero_padded_month);
        if self.micro.is_some() {
            let m = self.micro.clone().unwrap();
            version.push('.');
            version.push_str(&m);
        }
        version
    }

    fn short_year_from_str(year: &str) -> Result<String, String> {
        let y = year.parse::<u8>().map_err(|e| e.to_string())?;
        match y {
            0..=99 => Ok(format!("{y}")),
            _ => Err(
                "invalid 2-digit year provided: expected a number between 0 and 99, got {year}"
                    .to_string(),
            ),
        }
    }

    fn zero_padded_month_from_str(month: &str) -> Result<String, String> {
        let m = month.parse::<u8>().map_err(|e| e.to_string())?;
        match m {
            1..=9 => Ok(format!("0{m}")),
            10..=12 => Ok(format!("{m}")),
            _ => Err(
                "invalid 2-digit month provided: expected a number between 1 and 12, got {month}"
                    .to_string(),
            ),
        }
    }

    fn micro_from_str(micro: Option<&str>) -> Result<Option<String>, String> {
        match micro {
            None => Ok(None),
            Some(m) => match m.parse::<u32>() {
                Ok(value) => {
                    if value == 0 {
                        Ok(None)
                    } else {
                        Ok(Some(m.to_string()))
                    }
                }
                Err(e) => Err(e.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("24.01", true)]
    #[case("24.01.1", true)]
    #[case("4.01", true)]
    #[case("24", false)]
    #[case("24.01.1.1", false)]
    #[case("24.0", false)]
    #[case("0.0", false)]
    #[case("024.012.1", true)]
    #[case("24.012.1", true)]
    fn test_parse_ubuntu_calver(#[case] version: &str, #[case] is_ok: bool) {
        let res = Calver::try_from_ubuntu(version);
        assert_eq!(res.is_ok(), is_ok);
    }

    #[rstest]
    #[case("24.1", "23.1", true)]
    #[case("24.2", "23.1", true)]
    #[case("23.1", "24.1", false)]
    fn test_compare_ubuntu_calver(
        #[case] version: &str,
        #[case] other: &str,
        #[case] is_greater: bool,
    ) {
        let c1 = Calver::try_from_ubuntu(version).unwrap();
        let c2 = Calver::try_from_ubuntu(other).unwrap();
        assert_eq!(c1 > c2, is_greater);
    }

    #[rstest]
    #[case("24.1", "24.1")]
    #[case("24.1", "24.1.0")]
    #[case("4.1", "04.01.0")]
    fn test_equality_ubuntu_calver(#[case] version: &str, #[case] other: &str) {
        let c1 = Calver::try_from_ubuntu(version).unwrap();
        let c2 = Calver::try_from_ubuntu(other).unwrap();
        assert_eq!(c1, c2);
    }
}

use crate::errors::Error;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AnyKind {
    Postgres,
    MySql,
    Sqlite,
}

impl FromStr for AnyKind {
    type Err = Error;

    fn from_str(url: &str) -> Result<Self, Self::Err> {
        match url {
            _ if url.starts_with("postgres://") || url.starts_with("postgresql://") => {
                Ok(AnyKind::Postgres)
            }

            _ if url.starts_with("mysql://") || url.starts_with("mariadb://") => Ok(AnyKind::MySql),

            _ if url.starts_with("sqlite://") => Ok(AnyKind::Sqlite),

            _ => Err(Error::Configuration(format!(
                "unrecognized database url: {url:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_url() {
        let kind = AnyKind::from_str("postgres://localhost/db").unwrap();
        assert_eq!(kind, AnyKind::Postgres);
    }

    #[test]
    fn test_postgresql_url() {
        let kind = AnyKind::from_str("postgresql://localhost/db").unwrap();
        assert_eq!(kind, AnyKind::Postgres);
    }

    #[test]
    fn test_mysql_url() {
        let kind = AnyKind::from_str("mysql://localhost/db").unwrap();
        assert_eq!(kind, AnyKind::MySql);
    }

    #[test]
    fn test_mariadb_url() {
        let kind = AnyKind::from_str("mariadb://localhost/db").unwrap();
        assert_eq!(kind, AnyKind::MySql);
    }

    #[test]
    fn test_sqlite_url() {
        let kind = AnyKind::from_str("sqlite://data.db").unwrap();
        assert_eq!(kind, AnyKind::Sqlite);
    }

    #[test]
    fn test_invalid_url() {
        let result = AnyKind::from_str("invalid://localhost/db");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_url() {
        let result = AnyKind::from_str("");
        assert!(result.is_err());
    }

    #[test]
    fn test_postgres_with_credentials() {
        let kind = AnyKind::from_str("postgres://user:pass@localhost/db").unwrap();
        assert_eq!(kind, AnyKind::Postgres);
    }

    #[test]
    fn test_mysql_with_port() {
        let kind = AnyKind::from_str("mysql://localhost:3306/plateau").unwrap();
        assert_eq!(kind, AnyKind::MySql);
    }

    #[test]
    fn test_sqlite_file_path() {
        let kind = AnyKind::from_str("sqlite:///path/to/plateau.db").unwrap();
        assert_eq!(kind, AnyKind::Sqlite);
    }

    #[test]
    fn test_anykind_equality() {
        assert_eq!(AnyKind::Postgres, AnyKind::Postgres);
        assert_ne!(AnyKind::Postgres, AnyKind::MySql);
        assert_ne!(AnyKind::MySql, AnyKind::Sqlite);
    }

    #[test]
    fn test_anykind_clone() {
        let kind1 = AnyKind::Postgres;
        let kind2 = kind1.clone();
        assert_eq!(kind1, kind2);
    }
}


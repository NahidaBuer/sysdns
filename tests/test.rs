#[cfg(test)]
mod tests {
    use sysdns::SysDNS;

    #[test]
    fn test_support() {
        assert!(SysDNS::is_support());
    }

    #[test]
    fn test_get() {
        SysDNS::get_system_dns().unwrap();
    }

    #[test]
    fn test_enable() {
        let mut sysdns = SysDNS {
            enable: true,
            server: "192.168.200.1".into(),
        };
        sysdns.set_system_dns().unwrap();

        let cur_dns = SysDNS::get_system_dns().unwrap();

        assert_eq!(cur_dns, sysdns);

        sysdns.enable = false;
        sysdns.server = "Empty".to_owned();
        sysdns.set_system_dns().unwrap();

        let current = SysDNS::get_system_dns().unwrap();
        assert_eq!(current, sysdns);
    }
}

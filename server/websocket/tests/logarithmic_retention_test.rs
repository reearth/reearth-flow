#[cfg(test)]
mod logarithmic_retention_tests {
    use std::collections::HashSet;

    #[test]
    fn test_first_zero_bit() {
        fn first_zero_bit(x: u32) -> u32 {
            (x + 1) & !x
        }

        assert_eq!(first_zero_bit(0), 1); 
        assert_eq!(first_zero_bit(1), 2); 
        assert_eq!(first_zero_bit(2), 1); 
        assert_eq!(first_zero_bit(3), 4); 
        assert_eq!(first_zero_bit(4), 1); 
        assert_eq!(first_zero_bit(5), 2); 
        assert_eq!(first_zero_bit(7), 8); 
        assert_eq!(first_zero_bit(8), 1); 
        assert_eq!(first_zero_bit(15), 16); 
        assert_eq!(first_zero_bit(16), 1); 
        assert_eq!(first_zero_bit(0xFFFF), 0x10000); 
    }

    #[test]
    fn test_logarithmic_retention_algorithm() {
        fn first_zero_bit(x: u32) -> u32 {
            (x + 1) & !x
        }

        let density_shift = 1; 
        let mut retained = HashSet::new();
        
        for n in 1..=100 {
            retained.insert(n);
            
            if n > 0 {
                let bit = first_zero_bit(n);
                let delete_offset = bit << density_shift;
                if delete_offset < n {
                    let to_delete = n - delete_offset;
                    retained.remove(&to_delete);
                }
            }
        }

        println!("Retained snapshots (density_shift={}): {:?}", density_shift, retained);
        println!("Retained {} out of 100 snapshots", retained.len());
        
        let expected_count = 2.0 * 100_f64.log2();
        println!("Theoretical count: approximately {}", expected_count);
        
        assert!(retained.len() <= 100); 
        assert!(retained.len() as f64 <= expected_count * 1.5); 
        
        assert!(retained.contains(&100));
    }

    #[test]
    fn test_density_parameter_impact() {
        fn first_zero_bit(x: u32) -> u32 {
            (x + 1) & !x
        }
        
        for density_shift in 0..=4 {
            let mut retained = HashSet::new();
            
            for n in 1..=200 {
                retained.insert(n);
                
                if n > 0 {
                    let bit = first_zero_bit(n);
                    let delete_offset = bit << density_shift;
                    if delete_offset < n {
                        let to_delete = n - delete_offset;
                        retained.remove(&to_delete);
                    }
                }
            }
            
            println!(
                "density_shift={}: retained {} snapshots out of 200 (approximately {:.1}% of total)", 
                density_shift, 
                retained.len(), 
                retained.len() as f64 / 200.0 * 100.0
            );
            
        }
        
    }
    
    #[test]
    fn test_edge_cases() {
        fn first_zero_bit(x: u32) -> u32 {
            (x + 1) & !x
        }

        let mut retained = HashSet::new();
        retained.insert(0);
        
        let bit = first_zero_bit(0);
        let delete_offset = bit << 1; 
        
        assert_eq!(delete_offset, 2); 
        
        assert!(0 < delete_offset); 
        assert!(retained.contains(&0)); 
        
        let large_n = 1_000_000;
        let bit = first_zero_bit(large_n);
        
        assert!(bit > 0);
        assert!((large_n - (bit << 1)) < large_n);
    }
} 
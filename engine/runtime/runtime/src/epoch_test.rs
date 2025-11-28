#[cfg(test)]
mod tests {
    use crate::epoch::{Epoch, EpochCommonInfo, SourceTime};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn test_source_time_new() {
        let source_time = SourceTime::new(1000, 100);
        assert_eq!(source_time.millis_since_epoch, 1000);
        assert_eq!(source_time.accuracy, 100);
    }

    #[test]
    fn test_source_time_from_chrono() {
        use chrono::{TimeZone, Utc};
        let date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
        let source_time = SourceTime::from_chrono(&date, 1000);
        
        assert!(source_time.millis_since_epoch > 0);
        assert_eq!(source_time.accuracy, 1000);
    }

    #[test]
    fn test_source_time_elapsed_millis() {
        let now = SystemTime::now();
        let now_millis = now
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let source_time = SourceTime::new(now_millis - 5000, 100);
        let elapsed = source_time.elapsed_millis();
        
        assert!(elapsed.is_some());
        assert!(elapsed.unwrap() >= 5000);
    }

    #[test]
    fn test_source_time_elapsed_millis_accuracy() {
        let now = SystemTime::now();
        let now_millis = now
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let source_time = SourceTime::new(now_millis - 1500, 1000);
        let elapsed = source_time.elapsed_millis();
        
        assert!(elapsed.is_some());
    }

    #[test]
    fn test_source_time_equality() {
        let time1 = SourceTime::new(1000, 100);
        let time2 = SourceTime::new(1000, 100);
        let time3 = SourceTime::new(2000, 100);
        
        assert_eq!(time1, time2);
        assert_ne!(time1, time3);
    }

    #[test]
    fn test_source_time_clone() {
        let time1 = SourceTime::new(5000, 500);
        let time2 = time1.clone();
        
        assert_eq!(time1, time2);
    }

    #[test]
    fn test_epoch_new() {
        let instant = SystemTime::now();
        let epoch = Epoch::new(1, instant);
        
        assert_eq!(epoch.common_info.id, 1);
        assert_eq!(epoch.decision_instant, instant);
        assert!(epoch.source_time.is_none());
    }

    #[test]
    fn test_epoch_with_source_time() {
        let instant = SystemTime::now();
        let source_time = SourceTime::new(1000, 100);
        let epoch = Epoch::new(1, instant).with_source_time(source_time);
        
        assert_eq!(epoch.common_info.id, 1);
        assert!(epoch.source_time.is_some());
        assert_eq!(epoch.source_time.unwrap(), source_time);
    }

    #[test]
    fn test_epoch_clone() {
        let instant = SystemTime::now();
        let source_time = SourceTime::new(2000, 200);
        let epoch1 = Epoch::new(5, instant).with_source_time(source_time);
        let epoch2 = epoch1.clone();
        
        assert_eq!(epoch1.common_info.id, epoch2.common_info.id);
        assert_eq!(epoch1.source_time, epoch2.source_time);
    }

    #[test]
    fn test_epoch_common_info() {
        let info = EpochCommonInfo { id: 42 };
        assert_eq!(info.id, 42);
    }

    #[test]
    fn test_epoch_common_info_clone() {
        let info1 = EpochCommonInfo { id: 123 };
        let info2 = info1.clone();
        
        assert_eq!(info1.id, info2.id);
    }

    #[test]
    fn test_epoch_sequential_ids() {
        let instant = SystemTime::now();
        let epoch1 = Epoch::new(1, instant);
        let epoch2 = Epoch::new(2, instant);
        let epoch3 = Epoch::new(3, instant);
        
        assert_eq!(epoch1.common_info.id, 1);
        assert_eq!(epoch2.common_info.id, 2);
        assert_eq!(epoch3.common_info.id, 3);
    }

    #[test]
    fn test_source_time_future_check() {
        let future = SystemTime::now() + Duration::from_secs(3600);
        let future_millis = future
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let source_time = SourceTime::new(future_millis, 1000);
        let elapsed = source_time.elapsed_millis();
        
        assert!(elapsed.is_none());
    }

    #[test]
    fn test_source_time_accuracy_variations() {
        let base_millis = 10000;
        
        let time_1ms = SourceTime::new(base_millis, 1);
        let time_100ms = SourceTime::new(base_millis, 100);
        let time_1sec = SourceTime::new(base_millis, 1000);
        
        assert_eq!(time_1ms.accuracy, 1);
        assert_eq!(time_100ms.accuracy, 100);
        assert_eq!(time_1sec.accuracy, 1000);
    }
}


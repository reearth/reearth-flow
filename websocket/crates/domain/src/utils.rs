use std::cmp::min;

#[macro_export]
macro_rules! generate_id {
    ($prefix:expr) => {
        format!("{}{}", $prefix, uuid::Uuid::new_v4())
    };
}

#[inline]
pub fn calculate_diff(client_state: &[u8], server_state: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut diff = Vec::with_capacity(min(client_state.len(), server_state.len()));
    let mut i = 0;
    let mut j = 0;

    while i < client_state.len() && j < server_state.len() {
        if client_state[i] == server_state[j] {
            let start = i;
            while i < client_state.len()
                && j < server_state.len()
                && client_state[i] == server_state[j]
                && i - start < 255
            {
                i += 1;
                j += 1;
            }
            diff.push(3);
            diff.push((i - start) as u8);
        } else {
            diff.push(2);
            diff.push(server_state[j]);
            i += 1;
            j += 1;
        }
    }

    while j < server_state.len() {
        diff.push(0);
        diff.push(server_state[j]);
        j += 1;
    }

    while i < client_state.len() {
        diff.push(1);
        i += 1;
    }

    (diff, server_state.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_diff() {
        // Test case 1: Identical states
        let client_state = vec![1, 2, 3, 4, 5];
        let server_state = vec![1, 2, 3, 4, 5];
        let (diff, result_server_state) = calculate_diff(&client_state, &server_state);
        assert_eq!(diff, vec![3, 5]);
        assert_eq!(result_server_state, server_state);

        // Test case 2: Server state has additional data
        let client_state = vec![1, 2, 3];
        let server_state = vec![1, 2, 3, 4, 5];
        let (diff, result_server_state) = calculate_diff(&client_state, &server_state);
        assert_eq!(diff, vec![3, 3, 0, 4, 0, 5]);
        assert_eq!(result_server_state, server_state);

        // Test case 3: Client state has additional data
        let client_state = vec![1, 2, 3, 4, 5];
        let server_state = vec![1, 2, 3];
        let (diff, result_server_state) = calculate_diff(&client_state, &server_state);
        assert_eq!(diff, vec![3, 3, 1, 1]);
        assert_eq!(result_server_state, server_state);

        // Test case 4: Different states
        let client_state = vec![1, 2, 3, 4, 5];
        let server_state = vec![1, 2, 6, 7, 8];
        let (diff, result_server_state) = calculate_diff(&client_state, &server_state);
        assert_eq!(diff, vec![3, 2, 2, 6, 2, 7, 2, 8]);
        assert_eq!(result_server_state, server_state);

        // Test case 5: Empty states
        let client_state = vec![];
        let server_state = vec![];
        let (diff, result_server_state) = calculate_diff(&client_state, &server_state);
        assert_eq!(diff, vec![]);
        assert_eq!(result_server_state, server_state);
    }
}

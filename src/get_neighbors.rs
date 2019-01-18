use crate::render_to_prometheus::RenderToPrometheus;
use std::fmt::Write;

#[derive(Debug, Deserialize)]
pub struct Neighbors {
    #[serde(rename = "neighbors")]
    neighbors: Vec<Neighbor>,
    #[serde(rename = "duration")]
    duration: u64,
}

#[derive(Debug, Deserialize)]
pub struct Neighbor {
    #[serde(rename = "address")]
    address: String,
    #[serde(rename = "numberOfAllTransactions")]
    number_of_all_transactions: u64,
    #[serde(rename = "numberOfRandomTransactionRequests")]
    number_of_random_transaction_requests: u64,
    #[serde(rename = "numberOfNewTransactions")]
    number_of_new_transactions: u64,
    #[serde(rename = "numberOfInvalidTransactions")]
    number_of_invalid_transactions: u64,
    #[serde(rename = "numberOfStaleTransactions")]
    number_of_stale_transactions: u64,
    #[serde(rename = "numberOfSentTransactions")]
    number_of_sent_transactions: u64,
    #[serde(rename = "connectionType")]
    connection_type: String,
}

impl RenderToPrometheus for Neighbors {
    fn render(&self) -> String {
        let mut s = String::with_capacity(1024 * 4);
        let mut cnt_active = 0;

        s.push_str(
            "\n# HELP iota_neighbors_new_transactions New transactions by neighbor
# TYPE iota_neighbors_new_transactions gauge\n",
        );
        for n in &self.neighbors {
            write!(
                &mut s,
                "iota_neighbors_new_transactions{{id=\"{}\"}} {}\n",
                n.address, n.number_of_new_transactions
            )
            .unwrap();

            if n.number_of_all_transactions > 0 {
                cnt_active += 1;
            }
        }

        s.push_str(
            "\n# HELP iota_neighbors_random_transactions Random transactions by neighbor
# TYPE iota_neighbors_random_transactions gauge\n",
        );
        for n in &self.neighbors {
            write!(
                &mut s,
                "iota_neighbors_random_transactions{{id=\"{}\"}} {}\n",
                n.address, n.number_of_random_transaction_requests
            )
            .unwrap();
        }

        s.push_str(
            "\n# HELP iota_neighbors_all_transactions All transactions by neighbor
# TYPE iota_neighbors_all_transactions gauge\n",
        );
        for n in &self.neighbors {
            write!(
                &mut s,
                "iota_neighbors_all_transactions{{id=\"{}\"}} {}\n",
                n.address, n.number_of_all_transactions
            )
            .unwrap();
        }

        s.push_str(
            "\n# HELP iota_neighbors_invalid_transactions Invalid transactions by neighbor
# TYPE iota_neighbors_invalid_transactions gauge\n",
        );
        for n in &self.neighbors {
            write!(
                &mut s,
                "iota_neighbors_invalid_transactions{{id=\"{}\"}} {}\n",
                n.address, n.number_of_invalid_transactions
            )
            .unwrap();
        }

        s.push_str(
            "\n# HELP iota_neighbors_sent_transactions Transactions sent to neighbor
# TYPE iota_neighbors_sent_transactions gauge\n",
        );
        for n in &self.neighbors {
            write!(
                &mut s,
                "iota_neighbors_sent_transactions{{id=\"{}\"}} {}\n",
                n.address, n.number_of_sent_transactions
            )
            .unwrap();
        }

        write!(
            &mut s,
            "\n# HELP iota_neighbors_active_neighbors Number of neighbors who are active
# TYPE iota_neighbors_active_neighbors gauge
iota_neighbors_active_neighbors {}\n",
            cnt_active
        )
        .unwrap();

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let s = "{
                  \"neighbors\": [
                    {
                      \"address\": \"CANTUN.mindflavor.it:14600\",
                      \"numberOfAllTransactions\": 0,
                      \"numberOfRandomTransactionRequests\": 0,
                      \"numberOfNewTransactions\": 0,
                      \"numberOfInvalidTransactions\": 0,
                      \"numberOfStaleTransactions\": 0,
                      \"numberOfSentTransactions\": 2138813,
                      \"connectionType\": \"udp\"
                    },
                    {
                      \"address\": \"185.144.100.110:14600\",
                      \"numberOfAllTransactions\": 1264786,
                      \"numberOfRandomTransactionRequests\": 44451,
                      \"numberOfNewTransactions\": 85689,
                      \"numberOfInvalidTransactions\": 0,
                      \"numberOfStaleTransactions\": 47690,
                      \"numberOfSentTransactions\": 1306122,
                      \"connectionType\": \"udp\"
                    },
                    {
                      \"address\": \"ume.iotanode.jp:14600\",
                      \"numberOfAllTransactions\": 413902,
                      \"numberOfRandomTransactionRequests\": 24760,
                      \"numberOfNewTransactions\": 103972,
                      \"numberOfInvalidTransactions\": 0,
                      \"numberOfStaleTransactions\": 41500,
                      \"numberOfSentTransactions\": 444715,
                      \"connectionType\": \"udp\"
                    },
                    {
                      \"address\": \"h2799399.stratoserver.net:14600\",
                      \"numberOfAllTransactions\": 430123,
                      \"numberOfRandomTransactionRequests\": 14978,
                      \"numberOfNewTransactions\": 31237,
                      \"numberOfInvalidTransactions\": 0,
                      \"numberOfStaleTransactions\": 225733,
                      \"numberOfSentTransactions\": 274775,
                      \"connectionType\": \"udp\"
                    },
                    {
                      \"address\": \"static.150.12.69.159.clients.your-server.de:14600\",
                      \"numberOfAllTransactions\": 108907,
                      \"numberOfRandomTransactionRequests\": 2550,
                      \"numberOfNewTransactions\": 33199,
                      \"numberOfInvalidTransactions\": 0,
                      \"numberOfStaleTransactions\": 9253,
                      \"numberOfSentTransactions\": 85649,
                      \"connectionType\": \"udp\"
                    }
                  ],
                  \"duration\": 0
                }";
        let dresp: Neighbors = serde_json::from_str(s).unwrap();

        assert_eq!(dresp.neighbors.len(), 5);
        assert_eq!(
            dresp.neighbors[4].address,
            "static.150.12.69.159.clients.your-server.de:14600"
        );
        assert_eq!(
            dresp.neighbors[3].number_of_random_transaction_requests,
            14_978
        );
    }
}

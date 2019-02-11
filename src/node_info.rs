use crate::render_to_prometheus::RenderToPrometheus;

#[derive(Debug, Deserialize)]
pub struct NodeInfo {
    #[serde(rename = "jreAvailableProcessors")]
    jre_available_processors: u64,
    #[serde(rename = "jreFreeMemory")]
    jre_free_memory: u64,
    #[serde(rename = "jreMaxMemory")]
    jre_max_memory: u64,
    #[serde(rename = "jreTotalMemory")]
    jre_total_memory: u64,
    #[serde(rename = "latestMilestoneIndex")]
    latest_milestone_index: u64,
    #[serde(rename = "latestSolidSubtangleMilestoneIndex")]
    latest_solid_subtangle_milestone_index: u64,
    #[serde(rename = "milestoneStartIndex")]
    milestone_start_index: u64,
    #[serde(rename = "lastSnapshottedMilestoneIndex")]
    last_snapshotted_milestone_index: u64,
    #[serde(rename = "neighbors")]
    neighbors: u32,
    #[serde(rename = "packetsQueueSize")]
    packet_queue_size: u64,
    #[serde(rename = "time")]
    time: u128,
    #[serde(rename = "tips")]
    tips: u64,
    #[serde(rename = "transactionsToRequest")]
    transactions_to_request: u64,
    #[serde(rename = "duration")]
    duration: u64,
}

impl RenderToPrometheus for NodeInfo {
    fn render(&self) -> String {
        format!("# HELP iota_node_info_total_transactions_queued Total open txs at the interval
# TYPE iota_node_info_total_transactions_queued gauge
iota_node_info_total_transactions_queued {}

# HELP iota_node_info_total_tips Total tips at the interval
# TYPE iota_node_info_total_tips gauge
iota_node_info_total_tips {}

# HELP iota_node_info_total_neighbors Total neighbors at the interval
# TYPE iota_node_info_total_neighbors gauge
iota_node_info_total_neighbors {}

# HELP iota_node_info_latest_milestone Tangle milestone at the interval
# TYPE iota_node_info_latest_milestone gauge
iota_node_info_latest_milestone {}

# HELP iota_node_info_latest_subtangle_milestone Subtangle milestone at the interval
# TYPE iota_node_info_latest_subtangle_milestone gauge
iota_node_info_latest_subtangle_milestone {}

# HELP iota_node_milestone_start_index Milestone start index
# TYPE iota_node_milestone_start_index gauge
iota_node_milestone_start_index {}

# HELP iota_node_info_latest_snapshotted_milestone Snapshotted milestone at the interval
# TYPE iota_node_info_latest_snapshotted_milestone gauge
iota_node_info_latest_snapshotted_milestone {}

# HELP iota_node_info_packet_queue_size Packet queue size
# TYPE iota_node_info_packet_queue_size gauge
iota_node_info_packet_queue_size {}

# HELP iota_node_info_jre_free_memory JRE free memory
# TYPE iota_node_info_jre_free_memory gauge
iota_node_info_jre_free_memory {}

# HELP iota_node_info_jre_max_memory JRE max memory
# TYPE iota_node_info_jre_max_memory gauge
iota_node_info_jre_max_memory {}

# HELP iota_node_info_jre_total_memory JRE total memory
# TYPE iota_node_info_jre_total_memory gauge
iota_node_info_jre_total_memory {}
         ",
        self.transactions_to_request, 
        self.tips,
        self.neighbors,
        self.latest_milestone_index,
        self.latest_solid_subtangle_milestone_index,
        self.milestone_start_index,
        self.last_snapshotted_milestone_index,
        self.packet_queue_size,
        self.jre_free_memory,
        self.jre_max_memory,
        self.jre_total_memory
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let s = 
            "{
              \"appName\": \"IRI\",
              \"appVersion\": \"1.6.0-RELEASE\",
              \"jreAvailableProcessors\": 4,
              \"jreFreeMemory\": 738844856,
              \"jreVersion\": \"1.8.0_191\",
              \"jreMaxMemory\": 3817865216,
              \"jreTotalMemory\": 1956642816,
              \"latestMilestone\": \"BSGKJQRZZLZKHGVYFIAYEPMKJNDQTKRAQQTVXEVZUEUJVLWICWFESOWD9OQUHLVJYKTCCCAMBXRMA9999\",
              \"latestMilestoneIndex\": 969299,
              \"latestSolidSubtangleMilestone\": \"BSGKJQRZZLZKHGVYFIAYEPMKJNDQTKRAQQTVXEVZUEUJVLWICWFESOWD9OQUHLVJYKTCCCAMBXRMA9999\",
              \"latestSolidSubtangleMilestoneIndex\": 969299,
              \"milestoneStartIndex\": 933211,
              \"lastSnapshottedMilestoneIndex\": 968833,
              \"neighbors\": 3,
              \"packetsQueueSize\": 0,
              \"time\": 1547653117673,
              \"tips\": 3687,
              \"transactionsToRequest\": 39,
              \"features\": [
                \"snapshotPruning\",
                \"dnsRefresher\",
                \"zeroMessageQueue\",
                \"tipSolidification\"
              ],
              \"coordinatorAddress\": \"KPWCHICGJZXKE9GSUDXZYUAPLHAKAHYHDXNPHENTERYMMBQOPSQIDENXKLKCEYCPVTZQLEEJVYJZV9BWU\",
              \"duration\": 0
            }";
        let dresp:NodeInfo  = serde_json::from_str(s).unwrap();

        assert_eq!(dresp.transactions_to_request, 39);
        assert_eq!(dresp.jre_free_memory, 738844856);
        assert_eq!(dresp.neighbors, 3);
    }
}

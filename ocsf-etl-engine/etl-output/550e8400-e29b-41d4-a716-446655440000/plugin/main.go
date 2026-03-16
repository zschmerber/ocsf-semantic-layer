package main

import (
    "github.com/ocsf/tangent/sdk"
)

// DnsActivity OCSF event class (UID: 4003, Category: 4)
type DnsActivity struct {
    ClassUID    int    `json:"class_uid"`
    CategoryUID int    `json:"category_uid"`
    // Field mappings follow
}

func ClassUID() int {
    return 4003
}

func CategoryUID() int {
    return 4
}

func Transform(record map[string]interface{}) *DnsActivity {
    event := &DnsActivity{
        ClassUID:    4003,
        CategoryUID: 4,
    }
    event.QueryHostname = record["hostname"]
    event.SrcEndpointIp = record["src_ip"]
    event.DstEndpointIp = record["dst_ip"]
    return event
}

func init() {
    sdk.RegisterPlugin("dns_activity_plugin", Transform)
}

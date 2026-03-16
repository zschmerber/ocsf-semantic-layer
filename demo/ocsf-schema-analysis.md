# OCSF Schema v1.6.0 Analysis for Semantic Layer Generation

## Overview

- **Categories**: 8
- **Classes (Event Types)**: 82
- **Objects (Reusable Structures)**: 167
- **Profiles**: 12
- **Schema Version**: 1.6.0

## Categories

### 1. System Activity (system)
System Activity events.

### 2. Findings (findings)
Findings events report findings, detections, and possible resolutions of malware, anomalies, or other actions performed by security products.

### 3. Identity & Access Management (iam)
Identity & Access Management (IAM) events relate to the supervision of the system's authentication and access control model. Examples of such events are the success or failure of authentication, granting of authority, password change, entity change, privileged use etc.

### 4. Network Activity (network)
Network Activity events.

### 5. Discovery (discovery)
Discovery events report the existence and state of devices, files, configurations, processes, registry keys, and other objects.

### 6. Application Activity (application)
Application Activity events report detailed information about the behavior of applications and services.

### 7. Remediation (remediation)
Remediation events report the results of remediation commands targeting files, processes, and other objects.

### 8. Unmanned Systems (unmanned_systems)
Unmanned Systems events report the activity, existence, and/or state of unmanned systems for tracking, mission planning, and other related activities.

## Event Classes by Category

### System Activity (uid=1)

- **1001** file_activity: File System Activity (objects: actor, cloud, device, file)
- **1002** kernel_extension_activity: Kernel Extension Activity (objects: actor, cloud, device, driver)
- **1003** kernel_activity: Kernel Activity (objects: actor, cloud, device, kernel)
- **1004** memory_activity: Memory Activity (objects: actor, cloud, device, metadata)
- **1005** module_activity: Module Activity (objects: actor, cloud, device, metadata)
- **1006** scheduled_job_activity: Scheduled Job Activity (objects: cloud, device, job, metadata)
- **1007** process_activity: Process Activity (objects: actor, cloud, device, metadata)
- **1008** event_log_actvity: Event Log Activity (objects: actor, cloud, device, dst_endpoint)
- **1009** script_activity: Script Activity (objects: actor, cloud, device, metadata)
- **201001** registry_key_activity: Registry Key Activity (objects: actor, cloud, device, metadata)
- **201002** registry_value_activity: Registry Value Activity (objects: actor, cloud, device, metadata)
- **201003** windows_resource_activity: Windows Resource Activity (objects: actor, cloud, device, metadata)
- **201004** windows_service_activity: Windows Service Activity (objects: actor, cloud, device, metadata)

### Findings (uid=2)

- **2001** security_finding: Security Finding (objects: analytic, cloud, device, finding)
- **2002** vulnerability_finding: Vulnerability Finding (objects: cloud, finding_info, metadata, observables)
- **2003** compliance_finding: Compliance Finding (objects: cloud, compliance, finding_info, metadata)
- **2004** detection_finding: Detection Finding (objects: cloud, evidences, finding_info, metadata)
- **2005** incident_finding: Incident Finding (objects: cloud, device, finding_info_list, metadata)
- **2006** data_security_finding: Data Security Finding (objects: actor, cloud, data_security, database)
- **2007** application_security_posture_finding: Application Security Posture Finding (objects: application, cloud, compliance, finding_info)
- **2008** iam_analysis_finding: IAM Analysis Finding (objects: applications, cloud, finding_info, identity_activity_metrics)

### Identity & Access Management (uid=3)

- **3001** account_change: Account Change (objects: actor, cloud, device, metadata)
- **3002** authentication: Authentication (objects: actor, certificate, cloud, device)
- **3003** authorize_session: Authorize Session (objects: actor, cloud, device, group)
- **3004** entity_management: Entity Management (objects: actor, cloud, device, entity)
- **3005** user_access: User Access Management (objects: actor, cloud, device, metadata)
- **3006** group_management: Group Management (objects: actor, cloud, device, group)

### Network Activity (uid=4)

- **4001** network_activity: Network Activity (objects: cloud, connection_info, device, dst_endpoint)
- **4002** http_activity: HTTP Activity (objects: cloud, connection_info, device, dst_endpoint)
- **4003** dns_activity: DNS Activity (objects: answers, cloud, device, dst_endpoint)
- **4004** dhcp_activity: DHCP Activity (objects: cloud, connection_info, device, dst_endpoint)
- **4005** rdp_activity: RDP Activity (objects: cloud, connection_info, dst_endpoint, load_balancer)
- **4006** smb_activity: SMB Activity (objects: cloud, connection_info, device, dst_endpoint)
- **4007** ssh_activity: SSH Activity (objects: client_hassh, cloud, connection_info, device)
- **4008** ftp_activity: FTP Activity (objects: cloud, connection_info, device, dst_endpoint)
- **4009** email_activity: Email Activity (objects: cloud, device, dst_endpoint, email)
- **4010** network_file_activity: Network File Activity (objects: actor, cloud, device, dst_endpoint)
- **4011** email_file_activity: Email File Activity (objects: cloud, device, file, metadata)
- **4012** email_url_activity: Email URL Activity (objects: cloud, device, metadata, observables)
- **4013** ntp_activity: NTP Activity (objects: cloud, connection_info, device, dst_endpoint)
- **4014** tunnel_activity: Tunnel Activity (objects: cloud, device, dst_endpoint, load_balancer)

### Discovery (uid=5)

- **5001** inventory_info: Device Inventory Info (objects: cloud, device, metadata, observables)
- **5002** config_state: Device Config State (objects: cis_benchmark_result, cloud, device, metadata)
- **5003** user_inventory: User Inventory Info (objects: cloud, device, metadata, observables)
- **5004** patch_state: Operating System Patch State (objects: cloud, device, kb_article_list, metadata)
- **5006** kernel_object_query: Kernel Object Query (objects: cloud, device, kernel, metadata)
- **5007** file_query: File Query (objects: cloud, device, file, metadata)
- **5008** folder_query: Folder Query (objects: cloud, device, folder, metadata)
- **5009** admin_group_query: Admin Group Query (objects: cloud, device, group, metadata)
- **5010** job_query: Job Query (objects: cloud, device, job, metadata)
- **5011** module_query: Module Query (objects: cloud, device, metadata, module)
- **5012** network_connection_query: Network Connection Query (objects: cloud, connection_info, device, metadata)
- **5013** networks_query: Networks Query (objects: cloud, device, metadata, network_interfaces)
- **5014** peripheral_device_query: Peripheral Device Query (objects: cloud, device, metadata, observables)
- **5015** process_query: Process Query (objects: cloud, device, metadata, observables)
- **5016** service_query: Service Query (objects: cloud, device, metadata, observables)
- **5017** session_query: User Session Query (objects: cloud, device, metadata, observables)
- **5018** user_query: User Query (objects: cloud, device, metadata, observables)
- **5019** device_config_state_change: Device Config State Change (objects: cloud, device, metadata, observables)
- **5020** software_info: Software Inventory Info (objects: cloud, device, metadata, observables)
- **5021** osint_inventory_info: OSINT Inventory Info (objects: cloud, device, metadata, observables)
- **5022** startup_item_query: Startup Item Query (objects: cloud, device, metadata, observables)
- **5023** cloud_resources_inventory_info: Cloud Resources Inventory Info (objects: cloud, container, database, databucket)
- **5040** evidence_info: Live Evidence Info (objects: cloud, device, metadata, observables)
- **205004** registry_key_query: Registry Key Query (objects: cloud, device, metadata, observables)
- **205005** registry_value_query: Registry Value Query (objects: cloud, device, metadata, observables)
- **205019** prefetch_query: Prefetch Query (objects: cloud, device, metadata, observables)

### Application Activity (uid=6)

- **6001** web_resources_activity: Web Resources Activity (objects: cloud, device, dst_endpoint, http_request)
- **6002** application_lifecycle: Application Lifecycle (objects: app, cloud, device, metadata)
- **6003** api_activity: API Activity (objects: actor, api, cloud, device)
- **6004** web_resource_access_activity: Web Resource Access Activity (objects: cloud, device, http_request, metadata)
- **6005** datastore_activity: Datastore Activity (objects: actor, cloud, database, databucket)
- **6006** file_hosting: File Hosting Activity (objects: actor, cloud, device, dst_endpoint)
- **6007** scan_activity: Scan Activity (objects: cloud, device, metadata, observables)
- **6008** application_error: Application Error (objects: cloud, device, metadata, observables)

### Remediation (uid=7)

- **7001** remediation_activity: Remediation Activity (objects: cloud, countermeasures, device, metadata)
- **7002** file_remediation_activity: File Remediation Activity (objects: cloud, countermeasures, device, file)
- **7003** process_remediation_activity: Process Remediation Activity (objects: cloud, countermeasures, device, metadata)
- **7004** network_remediation_activity: Network Remediation Activity (objects: cloud, connection_info, countermeasures, device)

### Unmanned Systems (uid=8)

- **8001** drone_flights_activity: Drone Flights Activity (objects: cloud, connection_info, device, dst_endpoint)
- **8002** airborne_broadcast_activity: Airborne Broadcast Activity (objects: aircraft, cloud, connection_info, device)

## Key Objects for Semantic Entities

### Most Used Objects

- **policy**: used in 83 classes, required in 0 (HIGH)
- **actor**: used in 82 classes, required in 15 (HIGH)
- **api**: used in 82 classes, required in 1 (HIGH)
- **attack**: used in 82 classes, required in 0 (HIGH)
- **authorization**: used in 82 classes, required in 0 (HIGH)
- **cloud**: used in 82 classes, required in 81 (HIGH)
- **device**: used in 82 classes, required in 18 (HIGH)
- **enrichment**: used in 82 classes, required in 0 (HIGH)
- **firewall_rule**: used in 82 classes, required in 0 (HIGH)
- **malware**: used in 82 classes, required in 0 (HIGH)
- **malware_scan_info**: used in 82 classes, required in 0 (HIGH)
- **metadata**: used in 82 classes, required in 82 (HIGH)
- **observable**: used in 82 classes, required in 0 (HIGH)
- **osint**: used in 82 classes, required in 82 (HIGH)
- **fingerprint**: used in 82 classes, required in 0 (HIGH)
- **object**: used in 82 classes, required in 0 (HIGH)
- **network_endpoint**: used in 49 classes, required in 5 (HIGH)
- **network_connection_info**: used in 29 classes, required in 2 (HIGH)
- **tls**: used in 28 classes, required in 0 (HIGH)
- **network_proxy**: used in 27 classes, required in 0 (HIGH)

## Detailed Object Analysis

### user
Caption: User
Attributes: 21 total

Recommended: has_mfa, name, type_id, uid
Enums: risk_level_id, type_id
Nested objects: account->account, groups->group, ldap_person->ldap_person, org->organization, programmatic_credentials->programmatic_credential

### actor
Caption: Actor
Attributes: 8 total

Recommended: process, user
Nested objects: authorizations->authorization, idp->idp, process->process, session->session, user->user

### device
Caption: Device
Attributes: 63 total

Required: type_id
Recommended: container, hostname, instance_uid, interface_name, interface_uid, namespace_pid, owner, region, type, uid
Enums: risk_level_id, type_id
Nested objects: agent_list->agent, container->container, groups->group, hw_info->device_hw_info, image->image

### network_endpoint
Caption: Network Endpoint
Attributes: 29 total

Recommended: container, hostname, instance_uid, interface_name, interface_uid, ip, name, namespace_pid, owner, port
Enums: type_id
Nested objects: agent_list->agent, autonomous_system->autonomous_system, container->container, hw_info->device_hw_info, location->location

### file
Caption: File
Attributes: 46 total

Required: name, type_id
Recommended: data_classification, data_classifications, ext, hashes, path
Enums: confidentiality_id, drive_type_id, type_id
Nested objects: accessor->user, creator->user, data_classification->data_classification, data_classifications->data_classification, encryption_details->encryption_details

### process
Caption: Process
Attributes: 31 total

Recommended: cmd_line, container, cpid, created_time, file, group, name, namespace_pid, parent_process, pid
Enums: integrity_id
Nested objects: ancestry->process_entity, container->container, environment_variables->environment_variable, file->file, group->group

### cloud
Caption: Cloud
Attributes: 7 total

Required: provider
Recommended: region
Nested objects: account->account, org->organization

### account
Caption: Account
Attributes: 6 total

Recommended: name, type_id, uid
Enums: type_id
Nested objects: tags->key_value_object

## Common Enum Types

- **action_id**: used in 82 classes
- **activity_id**: used in 82 classes
- **category_uid**: used in 82 classes
- **class_uid**: used in 82 classes
- **confidence_id**: used in 82 classes
- **disposition_id**: used in 82 classes
- **risk_level_id**: used in 82 classes
- **severity_id**: used in 82 classes
- **status_id**: used in 82 classes
- **type_uid**: used in 82 classes
- **query_result_id**: used in 18 classes
- **impact_id**: used in 8 classes
- **priority_id**: used in 7 classes
- **verdict_id**: used in 7 classes
- **state_id**: used in 3 classes

## Semantic Layer Recommendations

### Tier 1: Core Entities (from Objects)
1. User - Identity information
2. Device - Endpoint/host info
3. Actor - Who performed action
4. NetworkEndpoint - Network source/dest
5. File - File system objects
6. Process - Running processes
7. Cloud - Cloud context
8. Account - Credentials

### Tier 2: Event Entities (from Classes)
Each class becomes a semantic entity with flattened attributes.

### Dimension Inference Rules
1. Required attrs -> dimensions
2. Enum attrs -> categorical dimensions
3. group:primary -> high priority
4. Observable attrs -> searchable

### Metric Inference Rules
1. count -> event count
2. duration -> time metric
3. size -> size metric
4. severity_id -> severity distribution
5. status_id -> success/failure rate

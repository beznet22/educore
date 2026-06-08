SET FOREIGN_KEY_CHECKS=0;
CREATE TABLE IF NOT EXISTS `absent_notification_time_setups` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `time_from` varchar(191) DEFAULT NULL,
  `time_to` varchar(191) DEFAULT NULL,
  `active_status` int(11) NOT NULL DEFAULT 1,
  `school_id` int(11) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `chat_block_users` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `block_by` bigint(20) unsigned NOT NULL,
  `block_to` bigint(20) unsigned NOT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `chat_conversations` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `from_id` bigint(20) unsigned DEFAULT NULL,
  `to_id` bigint(20) unsigned DEFAULT NULL,
  `message` text DEFAULT NULL,
  `status` tinyint(4) NOT NULL DEFAULT 0 COMMENT '0 for unread,1 for seen',
  `message_type` tinyint(4) NOT NULL DEFAULT 0 COMMENT '0- text message, 1- image, 2- pdf, 3- doc, 4- voice',
  `file_name` text DEFAULT NULL,
  `original_file_name` text DEFAULT NULL,
  `initial` tinyint(1) NOT NULL DEFAULT 0,
  `reply` bigint(20) unsigned DEFAULT NULL,
  `forward` bigint(20) unsigned DEFAULT NULL,
  `deleted_by_to` tinyint(1) NOT NULL DEFAULT 0,
  `deleted_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `chat_groups` (
  `id` char(36) NOT NULL,
  `name` varchar(191) NOT NULL,
  `description` varchar(191) DEFAULT NULL,
  `photo_url` varchar(191) DEFAULT NULL,
  `privacy` int(11) DEFAULT NULL,
  `read_only` tinyint(1) NOT NULL DEFAULT 0,
  `group_type` int(11) NOT NULL DEFAULT 1 COMMENT '1 => Open (Anyone can send message), 2 => Close (Only Admin can send message) ',
  `created_by` bigint(20) unsigned NOT NULL,
  `class_id` int(10) unsigned DEFAULT NULL,
  `section_id` int(10) unsigned DEFAULT NULL,
  `subject_id` int(10) unsigned DEFAULT NULL,
  `teacher_id` int(10) unsigned DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT NULL,
  `academic_id` int(10) unsigned DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `chat_groups_class_id_foreign` (`class_id`),
  KEY `chat_groups_section_id_foreign` (`section_id`),
  KEY `chat_groups_subject_id_foreign` (`subject_id`),
  KEY `chat_groups_teacher_id_foreign` (`teacher_id`),
  KEY `chat_groups_school_id_foreign` (`school_id`),
  KEY `chat_groups_academic_id_foreign` (`academic_id`),
  CONSTRAINT `chat_groups_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chat_groups_class_id_foreign` FOREIGN KEY (`class_id`) REFERENCES `sm_classes` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chat_groups_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chat_groups_section_id_foreign` FOREIGN KEY (`section_id`) REFERENCES `sm_sections` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chat_groups_subject_id_foreign` FOREIGN KEY (`subject_id`) REFERENCES `sm_subjects` (`id`) ON DELETE CASCADE,
  CONSTRAINT `chat_groups_teacher_id_foreign` FOREIGN KEY (`teacher_id`) REFERENCES `sm_staffs` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `chat_group_message_recipients` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `user_id` bigint(20) unsigned NOT NULL,
  `conversation_id` bigint(20) unsigned NOT NULL,
  `group_id` varchar(191) NOT NULL,
  `read_at` datetime DEFAULT NULL,
  `deleted_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `chat_group_message_removes` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `group_message_recipient_id` bigint(20) unsigned NOT NULL,
  `user_id` bigint(20) unsigned NOT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `chat_group_users` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `group_id` char(36) NOT NULL,
  `user_id` bigint(20) unsigned NOT NULL,
  `role` int(11) NOT NULL DEFAULT 1,
  `added_by` bigint(20) unsigned NOT NULL,
  `removed_by` bigint(20) unsigned DEFAULT NULL,
  `deleted_at` datetime DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `chat_invitations` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `from` int(10) unsigned NOT NULL,
  `to` int(10) unsigned NOT NULL,
  `status` tinyint(4) NOT NULL DEFAULT 0 COMMENT '0- pending, 1- connected, 2- blocked',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `chat_invitation_types` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `invitation_id` bigint(20) unsigned NOT NULL,
  `type` enum('one-to-one','group','class-teacher') NOT NULL DEFAULT 'one-to-one',
  `section_id` bigint(20) unsigned DEFAULT NULL,
  `class_teacher_id` bigint(20) unsigned DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `chat_statuses` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `user_id` bigint(20) unsigned NOT NULL,
  `status` tinyint(4) NOT NULL DEFAULT 0 COMMENT '0- inactive, 1- active, 2- away, 3- busy',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `custom_sms_settings` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `gateway_id` int(11) NOT NULL,
  `gateway_name` varchar(191) NOT NULL,
  `set_auth` varchar(191) DEFAULT NULL,
  `gateway_url` varchar(191) NOT NULL,
  `request_method` varchar(191) NOT NULL,
  `send_to_parameter_name` varchar(191) NOT NULL,
  `messege_to_parameter_name` varchar(191) NOT NULL,
  `param_key_1` varchar(191) DEFAULT NULL,
  `param_value_1` varchar(191) DEFAULT NULL,
  `param_key_2` varchar(191) DEFAULT NULL,
  `param_value_2` varchar(191) DEFAULT NULL,
  `param_key_3` varchar(191) DEFAULT NULL,
  `param_value_3` varchar(191) DEFAULT NULL,
  `param_key_4` varchar(191) DEFAULT NULL,
  `param_value_4` varchar(191) DEFAULT NULL,
  `param_key_5` varchar(191) DEFAULT NULL,
  `param_value_5` varchar(191) DEFAULT NULL,
  `param_key_6` varchar(191) DEFAULT NULL,
  `param_value_6` varchar(191) DEFAULT NULL,
  `param_key_7` varchar(191) DEFAULT NULL,
  `param_value_7` varchar(191) DEFAULT NULL,
  `param_key_8` varchar(191) DEFAULT NULL,
  `param_value_8` varchar(191) DEFAULT NULL,
  `school_id` int(11) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `notifications` (
  `id` char(36) NOT NULL,
  `type` varchar(191) NOT NULL,
  `notifiable_type` varchar(191) NOT NULL,
  `notifiable_id` bigint(20) unsigned NOT NULL,
  `data` text NOT NULL,
  `read_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `notifications_notifiable_type_notifiable_id_index` (`notifiable_type`,`notifiable_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sms_templates` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `type` varchar(191) NOT NULL COMMENT 'email, sms',
  `purpose` text NOT NULL,
  `subject` text NOT NULL,
  `body` longtext NOT NULL,
  `module` varchar(191) NOT NULL,
  `variable` text NOT NULL,
  `status` int(11) NOT NULL DEFAULT 1 COMMENT 'Enable & Disable',
  `school_id` int(10) unsigned DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sms_templates_school_id_foreign` (`school_id`),
  CONSTRAINT `sms_templates_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_complaints` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `complaint_by` varchar(191) DEFAULT NULL,
  `complaint_type` tinyint(4) DEFAULT NULL,
  `complaint_source` tinyint(4) DEFAULT NULL,
  `phone` varchar(191) DEFAULT NULL,
  `date` date DEFAULT NULL,
  `description` text DEFAULT NULL,
  `action_taken` varchar(191) DEFAULT NULL,
  `assigned` varchar(191) DEFAULT NULL,
  `file` varchar(191) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_complaints_school_id_foreign` (`school_id`),
  KEY `sm_complaints_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_complaints_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_complaints_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_contact_messages` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(191) DEFAULT NULL,
  `phone` varchar(191) DEFAULT NULL,
  `email` varchar(191) DEFAULT NULL,
  `subject` varchar(191) DEFAULT NULL,
  `message` text DEFAULT NULL,
  `view_status` tinyint(4) NOT NULL DEFAULT 0,
  `reply_status` tinyint(4) NOT NULL DEFAULT 0,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_contact_messages_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_contact_messages_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_email_settings` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `email_engine_type` varchar(191) DEFAULT NULL,
  `from_name` varchar(191) DEFAULT NULL,
  `from_email` varchar(191) DEFAULT NULL,
  `mail_driver` varchar(191) DEFAULT NULL,
  `mail_host` varchar(191) DEFAULT NULL,
  `mail_port` varchar(191) DEFAULT NULL,
  `mail_username` varchar(191) DEFAULT NULL,
  `mail_password` varchar(191) DEFAULT NULL,
  `mail_encryption` varchar(191) DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_email_settings_school_id_foreign` (`school_id`),
  KEY `sm_email_settings_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_email_settings_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_email_settings_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_email_sms_logs` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `title` varchar(191) DEFAULT NULL,
  `description` varchar(191) DEFAULT NULL,
  `send_date` date DEFAULT NULL,
  `send_through` varchar(191) DEFAULT NULL,
  `send_to` varchar(191) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_email_sms_logs_school_id_foreign` (`school_id`),
  KEY `sm_email_sms_logs_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_email_sms_logs_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_email_sms_logs_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_notifications` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `date` date DEFAULT NULL,
  `message` varchar(191) DEFAULT NULL,
  `url` varchar(191) DEFAULT NULL,
  `is_read` tinyint(4) NOT NULL DEFAULT 0,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `user_id` int(10) unsigned DEFAULT 1,
  `role_id` int(10) unsigned NOT NULL DEFAULT 1,
  `created_by` int(10) unsigned NOT NULL DEFAULT 1,
  `updated_by` int(10) unsigned NOT NULL DEFAULT 1,
  `school_id` int(10) unsigned NOT NULL DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_notifications_school_id_foreign` (`school_id`),
  KEY `sm_notifications_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_notifications_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_notifications_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_notification_settings` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `event` varchar(191) DEFAULT NULL,
  `destination` varchar(191) DEFAULT NULL COMMENT 'E=email, S=SMS, W=web, A=app',
  `recipient` varchar(191) DEFAULT NULL,
  `subject` varchar(191) DEFAULT NULL,
  `template` longtext DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  `shortcode` text DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_notification_settings_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_notification_settings_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_phone_call_logs` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(191) DEFAULT NULL,
  `phone` varchar(191) DEFAULT NULL,
  `date` date DEFAULT NULL,
  `description` text DEFAULT NULL,
  `next_follow_up_date` date DEFAULT NULL,
  `call_duration` varchar(100) DEFAULT NULL,
  `call_type` varchar(2) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_phone_call_logs_school_id_foreign` (`school_id`),
  KEY `sm_phone_call_logs_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_phone_call_logs_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_phone_call_logs_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_send_messages` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `message_title` varchar(200) DEFAULT NULL,
  `message_des` varchar(500) DEFAULT NULL,
  `notice_date` date DEFAULT NULL,
  `publish_on` date DEFAULT NULL,
  `message_to` varchar(200) DEFAULT NULL COMMENT 'message sent to these roles',
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_send_messages_school_id_foreign` (`school_id`),
  KEY `sm_send_messages_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_send_messages_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_send_messages_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_sms_gateways` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `gateway_name` varchar(255) DEFAULT NULL,
  `type` varchar(5) DEFAULT 'com',
  `clickatell_username` varchar(255) DEFAULT NULL,
  `clickatell_password` varchar(255) DEFAULT NULL,
  `clickatell_api_id` varchar(255) DEFAULT NULL,
  `twilio_account_sid` varchar(255) DEFAULT NULL,
  `twilio_authentication_token` varchar(255) DEFAULT NULL,
  `twilio_registered_no` varchar(255) DEFAULT NULL,
  `msg91_authentication_key_sid` varchar(255) DEFAULT NULL,
  `msg91_sender_id` varchar(255) DEFAULT NULL,
  `msg91_route` varchar(255) DEFAULT NULL,
  `msg91_country_code` varchar(255) DEFAULT NULL,
  `textlocal_username` varchar(255) DEFAULT NULL,
  `textlocal_hash` varchar(255) DEFAULT NULL,
  `textlocal_sender` varchar(255) DEFAULT NULL,
  `device_info` text DEFAULT NULL,
  `africatalking_username` varchar(255) DEFAULT NULL,
  `africatalking_api_key` varchar(255) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 0,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `gateway_type` varchar(191) DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_sms_gateways_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_sms_gateways_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `speech_sliders` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(191) DEFAULT NULL,
  `designation` varchar(191) DEFAULT NULL,
  `speech` text DEFAULT NULL,
  `image` varchar(191) DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `speech_sliders_school_id_foreign` (`school_id`),
  CONSTRAINT `speech_sliders_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


SET FOREIGN_KEY_CHECKS=1;

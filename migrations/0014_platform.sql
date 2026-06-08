SET FOREIGN_KEY_CHECKS=0;
CREATE TABLE IF NOT EXISTS `comments` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `text` text NOT NULL,
  `is_flagged` tinyint(1) NOT NULL DEFAULT 0,
  `type` varchar(256) DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT NULL,
  `academic_id` int(10) unsigned DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `comments_school_id_foreign` (`school_id`),
  KEY `comments_academic_id_foreign` (`academic_id`),
  CONSTRAINT `comments_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE SET NULL,
  CONSTRAINT `comments_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `comment_pivots` (
  `comment_id` bigint(20) unsigned DEFAULT NULL,
  `comment_tag_id` bigint(20) unsigned DEFAULT NULL,
  KEY `comment_pivots_comment_id_foreign` (`comment_id`),
  KEY `comment_pivots_comment_tag_id_foreign` (`comment_tag_id`),
  CONSTRAINT `comment_pivots_comment_id_foreign` FOREIGN KEY (`comment_id`) REFERENCES `comments` (`id`) ON DELETE SET NULL,
  CONSTRAINT `comment_pivots_comment_tag_id_foreign` FOREIGN KEY (`comment_tag_id`) REFERENCES `comment_tags` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `comment_tags` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `tag` varchar(256) NOT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `academic_id` int(10) unsigned DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `comment_tags_tag_unique` (`tag`),
  KEY `comment_tags_academic_id_foreign` (`academic_id`),
  CONSTRAINT `comment_tags_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `continents` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `code` varchar(191) NOT NULL,
  `name` varchar(191) NOT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `continents_school_id_foreign` (`school_id`),
  CONSTRAINT `continents_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `continets` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `code` varchar(255) DEFAULT NULL,
  `name` varchar(255) DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `continets_school_id_foreign` (`school_id`),
  CONSTRAINT `continets_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `countries` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `code` varchar(191) NOT NULL,
  `name` varchar(191) NOT NULL,
  `native` varchar(191) NOT NULL,
  `phone` varchar(191) NOT NULL,
  `continent` varchar(191) NOT NULL,
  `capital` varchar(191) NOT NULL,
  `currency` varchar(191) NOT NULL,
  `languages` varchar(191) NOT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `countries_school_id_foreign` (`school_id`),
  KEY `countries_academic_id_foreign` (`academic_id`),
  CONSTRAINT `countries_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `countries_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `infix_module_infos` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `module_id` int(11) DEFAULT NULL,
  `module_name` varchar(191) DEFAULT NULL,
  `parent_id` int(11) DEFAULT 0,
  `name` varchar(191) DEFAULT NULL,
  `is_saas` tinyint(4) NOT NULL DEFAULT 0,
  `route` varchar(191) DEFAULT NULL,
  `parent_route` varchar(191) DEFAULT NULL,
  `lang_name` varchar(191) DEFAULT NULL,
  `icon_class` varchar(191) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT NULL,
  `type` int(11) DEFAULT NULL COMMENT '1 for module, 2 for module link, 3 for module links crud',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `infix_module_infos_created_by_foreign` (`created_by`),
  KEY `infix_module_infos_updated_by_foreign` (`updated_by`),
  KEY `infix_module_infos_school_id_foreign` (`school_id`),
  CONSTRAINT `infix_module_infos_created_by_foreign` FOREIGN KEY (`created_by`) REFERENCES `users` (`id`) ON DELETE CASCADE,
  CONSTRAINT `infix_module_infos_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE,
  CONSTRAINT `infix_module_infos_updated_by_foreign` FOREIGN KEY (`updated_by`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `infix_module_managers` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(200) DEFAULT NULL,
  `email` varchar(200) DEFAULT NULL,
  `notes` varchar(255) DEFAULT NULL,
  `version` varchar(200) DEFAULT NULL,
  `update_url` varchar(200) DEFAULT NULL,
  `purchase_code` varchar(200) DEFAULT NULL,
  `checksum` varchar(200) DEFAULT NULL,
  `installed_domain` varchar(200) DEFAULT NULL,
  `is_default` tinyint(1) NOT NULL DEFAULT 0,
  `addon_url` varchar(191) DEFAULT NULL,
  `activated_date` date DEFAULT NULL,
  `lang_type` int(11) DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `languages` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `code` varchar(191) NOT NULL,
  `name` varchar(191) NOT NULL,
  `native` varchar(191) NOT NULL,
  `rtl` tinyint(4) NOT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 0,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `languages_school_id_foreign` (`school_id`),
  CONSTRAINT `languages_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `personal_access_tokens` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `tokenable_type` varchar(191) NOT NULL,
  `tokenable_id` bigint(20) unsigned NOT NULL,
  `name` varchar(191) NOT NULL,
  `token` varchar(64) NOT NULL,
  `abilities` text DEFAULT NULL,
  `last_used_at` timestamp NULL DEFAULT NULL,
  `expires_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `personal_access_tokens_token_unique` (`token`),
  KEY `personal_access_tokens_tokenable_type_tokenable_id_index` (`tokenable_type`,`tokenable_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `plugins` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(191) NOT NULL,
  `is_enable` tinyint(1) NOT NULL DEFAULT 0,
  `availability` varchar(191) NOT NULL DEFAULT 'both',
  `show_admin_panel` tinyint(1) NOT NULL DEFAULT 0,
  `show_website` tinyint(1) NOT NULL DEFAULT 1,
  `showing_page` varchar(191) NOT NULL DEFAULT 'all',
  `applicable_for` varchar(191) DEFAULT NULL,
  `position` varchar(191) DEFAULT NULL,
  `short_code` varchar(50) DEFAULT NULL,
  `school_id` int(10) unsigned NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `plugins_school_id_foreign` (`school_id`),
  CONSTRAINT `plugins_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `school_modules` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `modules` longtext DEFAULT NULL,
  `menus` longtext DEFAULT NULL,
  `module_name` varchar(191) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `updated_by` int(11) DEFAULT NULL,
  `school_id` int(10) unsigned NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `school_modules_school_id_foreign` (`school_id`),
  CONSTRAINT `school_modules_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_add_ons` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_amount_transfers` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `amount` int(11) DEFAULT NULL,
  `purpose` varchar(191) DEFAULT NULL,
  `from_payment_method` int(11) DEFAULT NULL,
  `from_bank_name` int(11) DEFAULT NULL,
  `to_payment_method` int(11) DEFAULT NULL,
  `to_bank_name` int(11) DEFAULT NULL,
  `transfer_date` date DEFAULT NULL,
  `active_status` tinyint(4) DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_amount_transfers_school_id_foreign` (`school_id`),
  KEY `sm_amount_transfers_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_amount_transfers_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_amount_transfers_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_base_groups` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(200) NOT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_base_groups_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_base_groups_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_chart_of_accounts` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `head` varchar(200) DEFAULT NULL,
  `type` varchar(1) DEFAULT NULL COMMENT 'E = expense, I = income',
  `active_status` int(11) DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_chart_of_accounts_school_id_foreign` (`school_id`),
  KEY `sm_chart_of_accounts_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_chart_of_accounts_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_chart_of_accounts_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_countries` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `code` varchar(255) DEFAULT NULL,
  `name` varchar(255) DEFAULT NULL,
  `native` varchar(255) DEFAULT NULL,
  `phone` varchar(255) DEFAULT NULL,
  `continent` varchar(255) DEFAULT NULL,
  `capital` varchar(255) DEFAULT NULL,
  `currency` varchar(255) DEFAULT NULL,
  `languages` varchar(255) DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_countries_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_countries_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;

CREATE TABLE IF NOT EXISTS `sm_courses` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `title` varchar(191) NOT NULL,
  `image` text NOT NULL,
  `category_id` int(11) NOT NULL,
  `overview` text DEFAULT NULL,
  `outline` text DEFAULT NULL,
  `prerequisites` text DEFAULT NULL,
  `resources` text DEFAULT NULL,
  `stats` text DEFAULT NULL,
  `active_status` int(11) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_courses_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_courses_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_course_categories` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `category_name` varchar(191) DEFAULT NULL,
  `category_image` text DEFAULT NULL,
  `school_id` bigint(20) unsigned NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_currencies` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(191) DEFAULT NULL,
  `code` varchar(191) DEFAULT NULL,
  `symbol` varchar(191) DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `currency_type` varchar(2) DEFAULT '2',
  `currency_position` varchar(2) DEFAULT '2',
  `space` tinyint(1) DEFAULT 1,
  `decimal_digit` int(11) DEFAULT NULL,
  `decimal_separator` varchar(1) DEFAULT NULL,
  `thousand_separator` varchar(191) DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_currencies_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_currencies_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_custom_fields` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `form_name` varchar(191) NOT NULL,
  `label` varchar(191) NOT NULL,
  `type` varchar(191) NOT NULL,
  `min_max_length` varchar(191) DEFAULT NULL,
  `min_max_value` varchar(191) DEFAULT NULL,
  `name_value` varchar(191) DEFAULT NULL,
  `width` varchar(191) DEFAULT NULL,
  `required` tinyint(4) DEFAULT NULL,
  `school_id` int(11) DEFAULT 1,
  `academic_id` int(11) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_custom_field_values` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `custom_field_id` bigint(20) unsigned NOT NULL,
  `entity_id` int(10) unsigned NOT NULL,
  `entity_type` varchar(191) NOT NULL,
  `field_value` text DEFAULT NULL,
  `school_id` int(11) DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_custom_field_values_field_foreign` (`custom_field_id`),
  CONSTRAINT `sm_custom_field_values_field_foreign` FOREIGN KEY (`custom_field_id`) REFERENCES `sm_custom_fields` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_expert_teachers` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `staff_id` tinyint(4) NOT NULL,
  `created_by` tinyint(4) DEFAULT NULL,
  `updated_by` tinyint(4) DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  `position` int(11) NOT NULL DEFAULT 0,
  PRIMARY KEY (`id`),
  KEY `sm_expert_teachers_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_expert_teachers_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_frontend_persmissions` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(255) DEFAULT NULL,
  `parent_id` int(11) NOT NULL DEFAULT 0,
  `is_published` int(11) NOT NULL DEFAULT 0,
  `school_id` int(10) unsigned DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_frontend_persmissions_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_frontend_persmissions_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_header_menu_managers` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `type` varchar(191) NOT NULL,
  `element_id` bigint(20) unsigned DEFAULT NULL,
  `title` varchar(191) DEFAULT NULL,
  `link` varchar(191) DEFAULT NULL,
  `parent_id` bigint(20) unsigned DEFAULT NULL,
  `position` int(10) unsigned NOT NULL DEFAULT 0,
  `show` tinyint(1) NOT NULL DEFAULT 0,
  `is_newtab` tinyint(1) NOT NULL DEFAULT 0,
  `theme` varchar(191) NOT NULL DEFAULT 'default',
  `school_id` int(10) unsigned DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_header_menu_managers_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_header_menu_managers_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_instructions` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `title` varchar(200) NOT NULL,
  `description` text NOT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_instructions_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_instructions_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_modules` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(191) NOT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `order` int(11) NOT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_modules_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_modules_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_module_links` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `module_id` int(10) unsigned DEFAULT NULL,
  `name` varchar(191) DEFAULT NULL,
  `route` varchar(191) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_module_links_module_id_foreign` (`module_id`),
  KEY `sm_module_links_created_by_foreign` (`created_by`),
  KEY `sm_module_links_updated_by_foreign` (`updated_by`),
  KEY `sm_module_links_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_module_links_created_by_foreign` FOREIGN KEY (`created_by`) REFERENCES `users` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_module_links_module_id_foreign` FOREIGN KEY (`module_id`) REFERENCES `sm_modules` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_module_links_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_module_links_updated_by_foreign` FOREIGN KEY (`updated_by`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_photo_galleries` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `parent_id` int(11) DEFAULT NULL,
  `name` varchar(191) DEFAULT NULL,
  `description` text DEFAULT NULL,
  `feature_image` varchar(191) DEFAULT NULL,
  `gallery_image` varchar(191) DEFAULT NULL,
  `is_publish` tinyint(1) NOT NULL DEFAULT 1,
  `position` int(11) NOT NULL DEFAULT 0,
  `school_id` int(10) unsigned DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_photo_galleries_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_photo_galleries_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_schools` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `school_name` varchar(200) DEFAULT NULL,
  `created_by` tinyint(4) NOT NULL DEFAULT 1,
  `updated_by` tinyint(4) NOT NULL DEFAULT 1,
  `email` varchar(200) DEFAULT NULL,
  `domain` varchar(191) NOT NULL DEFAULT 'school',
  `address` text DEFAULT NULL,
  `phone` varchar(20) DEFAULT NULL,
  `school_code` varchar(200) DEFAULT NULL,
  `is_email_verified` tinyint(1) NOT NULL DEFAULT 0,
  `starting_date` date DEFAULT NULL,
  `ending_date` date DEFAULT NULL,
  `package_id` int(11) DEFAULT NULL,
  `plan_type` varchar(200) DEFAULT NULL,
  `region` int(11) DEFAULT NULL,
  `contact_type` enum('yearly','monthly','once') DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1 COMMENT '1 approved, 0 pending',
  `is_enabled` varchar(20) NOT NULL DEFAULT 'yes' COMMENT 'yes=Login enable, no=Login disable',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_social_media_icons` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `url` varchar(191) DEFAULT NULL,
  `icon` varchar(191) DEFAULT NULL,
  `status` tinyint(4) NOT NULL DEFAULT 0 COMMENT '1 active, 0 inactive',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_social_media_icons_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_social_media_icons_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_time_zones` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `code` varchar(191) DEFAULT NULL,
  `time_zone` varchar(191) DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_to_dos` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `todo_title` varchar(191) DEFAULT NULL,
  `date` date DEFAULT NULL,
  `complete_status` varchar(191) DEFAULT 'P' COMMENT 'C for complete, N for not Complete, P Pending',
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_to_dos_school_id_foreign` (`school_id`),
  KEY `sm_to_dos_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_to_dos_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_to_dos_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_video_galleries` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(191) DEFAULT NULL,
  `description` text DEFAULT NULL,
  `video_link` text DEFAULT NULL,
  `is_publish` tinyint(1) NOT NULL DEFAULT 1,
  `position` int(11) NOT NULL DEFAULT 0,
  `school_id` int(10) unsigned DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_video_galleries_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_video_galleries_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_visitors` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(255) NOT NULL,
  `phone` varchar(255) DEFAULT NULL,
  `visitor_id` varchar(255) DEFAULT NULL,
  `no_of_person` int(11) DEFAULT NULL,
  `purpose` varchar(255) DEFAULT NULL,
  `date` date DEFAULT NULL,
  `in_time` varchar(255) DEFAULT NULL,
  `out_time` varchar(255) DEFAULT NULL,
  `file` varchar(255) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_visitors_school_id_foreign` (`school_id`),
  KEY `sm_visitors_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_visitors_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_visitors_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `users` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `full_name` varchar(192) DEFAULT NULL,
  `username` varchar(192) DEFAULT NULL,
  `phone_number` varchar(191) DEFAULT NULL,
  `email` varchar(192) DEFAULT NULL,
  `password` varchar(100) DEFAULT NULL,
  `usertype` varchar(210) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `random_code` text DEFAULT NULL,
  `notificationToken` text DEFAULT NULL,
  `remember_token` varchar(100) DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `language` varchar(191) DEFAULT 'en',
  `style_id` int(11) DEFAULT 1,
  `rtl_ltl` int(11) DEFAULT 2,
  `selected_session` int(11) DEFAULT 1,
  `created_by` int(11) DEFAULT 1,
  `updated_by` int(11) DEFAULT 1,
  `access_status` int(11) DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `role_id` int(10) unsigned DEFAULT NULL,
  `is_administrator` enum('yes','no') NOT NULL DEFAULT 'no',
  `is_registered` tinyint(4) NOT NULL DEFAULT 0,
  `device_token` text DEFAULT NULL,
  `stripe_id` varchar(191) DEFAULT NULL,
  `card_brand` varchar(191) DEFAULT NULL,
  `card_last_four` varchar(4) DEFAULT NULL,
  `verified` varchar(191) DEFAULT NULL,
  `trial_ends_at` timestamp NULL DEFAULT NULL,
  `wallet_balance` double(8,2) NOT NULL DEFAULT 0.00,
  PRIMARY KEY (`id`),
  KEY `users_school_id_foreign` (`school_id`),
  KEY `users_role_id_foreign` (`role_id`),
  CONSTRAINT `users_role_id_foreign` FOREIGN KEY (`role_id`) REFERENCES `infix_roles` (`id`) ON DELETE CASCADE,
  CONSTRAINT `users_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `user_otp_codes` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int(10) unsigned DEFAULT NULL,
  `otp_code` varchar(191) NOT NULL,
  `expired_time` varchar(200) DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `user_otp_codes_user_id_foreign` (`user_id`),
  CONSTRAINT `user_otp_codes_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `users` (`id`) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `video_uploads` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `title` varchar(191) NOT NULL,
  `description` text DEFAULT NULL,
  `youtube_link` varchar(191) NOT NULL,
  `class_id` int(11) NOT NULL,
  `section_id` int(11) NOT NULL,
  `created_by` int(11) NOT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `academic_id` int(10) unsigned DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `video_uploads_academic_id_foreign` (`academic_id`),
  KEY `video_uploads_school_id_foreign` (`school_id`),
  CONSTRAINT `video_uploads_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `video_uploads_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


SET FOREIGN_KEY_CHECKS=1;

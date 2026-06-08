SET FOREIGN_KEY_CHECKS=0;
CREATE TABLE IF NOT EXISTS `assign_permissions` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `permission_id` int(11) DEFAULT NULL,
  `role_id` int(10) unsigned DEFAULT NULL,
  `status` tinyint(1) NOT NULL DEFAULT 1,
  `menu_status` tinyint(1) NOT NULL DEFAULT 1,
  `saas_schools` text DEFAULT NULL,
  `created_by` int(10) unsigned NOT NULL DEFAULT 1,
  `updated_by` int(10) unsigned NOT NULL DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `assign_permissions_school_id_foreign` (`school_id`),
  CONSTRAINT `assign_permissions_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `infix_permission_assigns` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `module_id` int(11) DEFAULT NULL COMMENT ' module id, module link id, module link options id',
  `module_info` varchar(191) DEFAULT NULL,
  `role_id` int(10) unsigned DEFAULT NULL,
  `saas_schools` text DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `infix_permission_assigns_role_id_foreign` (`role_id`),
  KEY `infix_permission_assigns_school_id_foreign` (`school_id`),
  CONSTRAINT `infix_permission_assigns_role_id_foreign` FOREIGN KEY (`role_id`) REFERENCES `infix_roles` (`id`) ON DELETE CASCADE,
  CONSTRAINT `infix_permission_assigns_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `infix_roles` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(100) DEFAULT NULL,
  `type` varchar(191) NOT NULL DEFAULT 'System',
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_by` varchar(191) DEFAULT '1',
  `updated_by` varchar(191) DEFAULT '1',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  `is_saas` int(10) unsigned DEFAULT 0,
  PRIMARY KEY (`id`),
  KEY `infix_roles_school_id_foreign` (`school_id`),
  CONSTRAINT `infix_roles_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `permissions` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `module` varchar(191) DEFAULT NULL,
  `sidebar_menu` varchar(191) DEFAULT NULL,
  `old_id` int(11) DEFAULT NULL,
  `section_id` int(11) DEFAULT 1,
  `parent_id` int(11) DEFAULT 0,
  `name` varchar(191) DEFAULT NULL,
  `route` varchar(191) DEFAULT NULL,
  `parent_route` varchar(191) DEFAULT NULL,
  `type` int(11) DEFAULT NULL COMMENT '1 = menu, 2 = submenu, 3 = action',
  `lang_name` varchar(191) DEFAULT NULL,
  `icon` text DEFAULT NULL,
  `svg` text DEFAULT NULL,
  `status` tinyint(4) NOT NULL DEFAULT 1,
  `menu_status` tinyint(4) NOT NULL DEFAULT 1,
  `position` int(11) NOT NULL DEFAULT 1,
  `is_saas` tinyint(4) NOT NULL DEFAULT 0,
  `relate_to_child` tinyint(4) DEFAULT 0,
  `is_menu` tinyint(4) DEFAULT NULL,
  `is_admin` tinyint(4) DEFAULT 0,
  `is_teacher` tinyint(4) DEFAULT 0,
  `is_student` tinyint(4) DEFAULT 0,
  `is_parent` tinyint(4) DEFAULT 0,
  `is_alumni` tinyint(4) DEFAULT 0,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `permission_section` tinyint(4) DEFAULT NULL,
  `alternate_module` varchar(191) DEFAULT NULL,
  `user_id` int(11) DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `permissions_school_id_foreign` (`school_id`),
  CONSTRAINT `permissions_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `permission_sections` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(191) DEFAULT NULL,
  `position` int(11) NOT NULL DEFAULT 9999,
  `user_id` int(11) NOT NULL DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `saas` tinyint(4) NOT NULL DEFAULT 0,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `permission_sections_school_id_foreign` (`school_id`),
  CONSTRAINT `permission_sections_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `roles` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(100) DEFAULT NULL,
  `type` varchar(191) NOT NULL DEFAULT 'System',
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_by` varchar(191) DEFAULT '1',
  `updated_by` varchar(191) DEFAULT '1',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `roles_school_id_foreign` (`school_id`),
  CONSTRAINT `roles_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_module_permissions` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `dashboard_id` int(11) DEFAULT NULL,
  `name` varchar(191) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `sm_module_permissions_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_module_permissions_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_module_permission_assigns` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `module_id` int(10) unsigned DEFAULT NULL,
  `role_id` int(10) unsigned DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_module_permission_assigns_module_id_foreign` (`module_id`),
  KEY `sm_module_permission_assigns_role_id_foreign` (`role_id`),
  KEY `sm_module_permission_assigns_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_module_permission_assigns_module_id_foreign` FOREIGN KEY (`module_id`) REFERENCES `sm_module_permissions` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_module_permission_assigns_role_id_foreign` FOREIGN KEY (`role_id`) REFERENCES `roles` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_module_permission_assigns_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_role_permissions` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `module_link_id` int(10) unsigned DEFAULT NULL,
  `role_id` int(10) unsigned DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_role_permissions_module_link_id_foreign` (`module_link_id`),
  KEY `sm_role_permissions_role_id_foreign` (`role_id`),
  KEY `sm_role_permissions_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_role_permissions_module_link_id_foreign` FOREIGN KEY (`module_link_id`) REFERENCES `sm_module_links` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_role_permissions_role_id_foreign` FOREIGN KEY (`role_id`) REFERENCES `roles` (`id`) ON DELETE CASCADE ON UPDATE CASCADE,
  CONSTRAINT `sm_role_permissions_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `two_factor_settings` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `via_sms` tinyint(1) NOT NULL DEFAULT 0,
  `via_email` tinyint(1) NOT NULL DEFAULT 1,
  `for_student` tinyint(4) NOT NULL DEFAULT 2,
  `for_parent` tinyint(4) NOT NULL DEFAULT 3,
  `for_teacher` tinyint(4) NOT NULL DEFAULT 4,
  `for_staff` tinyint(4) NOT NULL DEFAULT 6,
  `for_admin` tinyint(4) NOT NULL DEFAULT 1,
  `expired_time` double(8,2) NOT NULL DEFAULT 300.00,
  `school_id` int(10) unsigned NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `two_factor_settings_school_id_foreign` (`school_id`),
  CONSTRAINT `two_factor_settings_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


SET FOREIGN_KEY_CHECKS=1;

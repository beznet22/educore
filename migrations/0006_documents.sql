SET FOREIGN_KEY_CHECKS=0;
CREATE TABLE IF NOT EXISTS `sm_form_downloads` (
  `id` bigint(20) unsigned NOT NULL AUTO_INCREMENT,
  `title` varchar(191) DEFAULT NULL,
  `short_description` varchar(200) DEFAULT NULL,
  `publish_date` date DEFAULT NULL,
  `link` varchar(191) DEFAULT NULL,
  `file` varchar(191) DEFAULT NULL,
  `show_public` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_form_downloads_school_id_foreign` (`school_id`),
  CONSTRAINT `sm_form_downloads_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_postal_dispatches` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `to_title` varchar(191) DEFAULT NULL,
  `from_title` varchar(191) DEFAULT NULL,
  `reference_no` varchar(191) DEFAULT NULL,
  `address` varchar(191) DEFAULT NULL,
  `date` date DEFAULT NULL,
  `note` text DEFAULT NULL,
  `file` varchar(191) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_postal_dispatches_school_id_foreign` (`school_id`),
  KEY `sm_postal_dispatches_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_postal_dispatches_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_postal_dispatches_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_postal_receives` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `from_title` varchar(191) DEFAULT NULL,
  `to_title` varchar(191) DEFAULT NULL,
  `reference_no` varchar(191) DEFAULT NULL,
  `address` varchar(191) DEFAULT NULL,
  `date` date DEFAULT NULL,
  `note` text DEFAULT NULL,
  `file` varchar(191) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_postal_receives_school_id_foreign` (`school_id`),
  KEY `sm_postal_receives_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_postal_receives_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_postal_receives_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


SET FOREIGN_KEY_CHECKS=1;

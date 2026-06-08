SET FOREIGN_KEY_CHECKS=0;
CREATE TABLE IF NOT EXISTS `sm_books` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `book_title` varchar(200) DEFAULT NULL,
  `book_number` varchar(200) DEFAULT NULL,
  `isbn_no` varchar(200) DEFAULT NULL,
  `publisher_name` varchar(200) DEFAULT NULL,
  `author_name` varchar(200) DEFAULT NULL,
  `rack_number` varchar(50) DEFAULT NULL,
  `quantity` int(11) DEFAULT 0,
  `book_price` int(11) DEFAULT NULL,
  `post_date` date DEFAULT NULL,
  `details` varchar(500) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `book_subject_id` int(10) unsigned DEFAULT NULL,
  `book_category_id` int(10) unsigned DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_books_book_category_id_foreign` (`book_category_id`),
  KEY `sm_books_school_id_foreign` (`school_id`),
  KEY `sm_books_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_books_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_books_book_category_id_foreign` FOREIGN KEY (`book_category_id`) REFERENCES `sm_book_categories` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_books_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_book_categories` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `category_name` varchar(200) DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_book_categories_school_id_foreign` (`school_id`),
  KEY `sm_book_categories_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_book_categories_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_book_categories_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_book_issues` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `quantity` int(11) DEFAULT NULL,
  `given_date` date DEFAULT NULL,
  `due_date` date DEFAULT NULL,
  `issue_status` varchar(191) DEFAULT NULL,
  `note` varchar(500) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `book_id` int(10) unsigned DEFAULT NULL,
  `member_id` int(10) unsigned DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_book_issues_book_id_foreign` (`book_id`),
  KEY `sm_book_issues_member_id_foreign` (`member_id`),
  KEY `sm_book_issues_school_id_foreign` (`school_id`),
  KEY `sm_book_issues_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_book_issues_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_book_issues_book_id_foreign` FOREIGN KEY (`book_id`) REFERENCES `sm_books` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_book_issues_member_id_foreign` FOREIGN KEY (`member_id`) REFERENCES `users` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_book_issues_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS `sm_library_members` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `member_ud_id` varchar(191) DEFAULT NULL,
  `active_status` tinyint(4) NOT NULL DEFAULT 1,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  `member_type` int(10) unsigned DEFAULT NULL,
  `student_staff_id` int(10) unsigned DEFAULT NULL,
  `created_by` int(10) unsigned DEFAULT 1,
  `updated_by` int(10) unsigned DEFAULT 1,
  `school_id` int(10) unsigned DEFAULT 1,
  `academic_id` int(10) unsigned DEFAULT 1,
  PRIMARY KEY (`id`),
  KEY `sm_library_members_member_type_foreign` (`member_type`),
  KEY `sm_library_members_student_staff_id_foreign` (`student_staff_id`),
  KEY `sm_library_members_school_id_foreign` (`school_id`),
  KEY `sm_library_members_academic_id_foreign` (`academic_id`),
  CONSTRAINT `sm_library_members_academic_id_foreign` FOREIGN KEY (`academic_id`) REFERENCES `sm_academic_years` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_library_members_member_type_foreign` FOREIGN KEY (`member_type`) REFERENCES `roles` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_library_members_school_id_foreign` FOREIGN KEY (`school_id`) REFERENCES `sm_schools` (`id`) ON DELETE CASCADE,
  CONSTRAINT `sm_library_members_student_staff_id_foreign` FOREIGN KEY (`student_staff_id`) REFERENCES `users` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


SET FOREIGN_KEY_CHECKS=1;

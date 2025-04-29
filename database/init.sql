-- 创建数据库
CREATE DATABASE IF NOT EXISTS questionnaire DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE questionnaire;

-- 创建用户表
CREATE TABLE IF NOT EXISTS users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    nickname VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(100),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
) ENGINE=InnoDB;

-- 创建问卷表
CREATE TABLE IF NOT EXISTS questionnaires (
    id INT AUTO_INCREMENT PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    is_public BOOLEAN DEFAULT FALSE,
    creator_id INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE CASCADE
) ENGINE=InnoDB;

-- 创建问题表
CREATE TABLE IF NOT EXISTS questions (
    id INT AUTO_INCREMENT PRIMARY KEY,
    questionnaire_id INT NOT NULL,
    title VARCHAR(255) NOT NULL,
    question_type VARCHAR(50) NOT NULL, -- text, radio, checkbox
    required BOOLEAN DEFAULT TRUE,
    display_order INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (questionnaire_id) REFERENCES questionnaires(id) ON DELETE CASCADE
) ENGINE=InnoDB;

-- 创建问题选项表
CREATE TABLE IF NOT EXISTS question_options (
    id INT AUTO_INCREMENT PRIMARY KEY,
    question_id INT NOT NULL,
    option_text VARCHAR(255) NOT NULL,
    display_order INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (question_id) REFERENCES questions(id) ON DELETE CASCADE
) ENGINE=InnoDB;

-- 创建问卷回答表
CREATE TABLE IF NOT EXISTS questionnaire_responses (
    id INT AUTO_INCREMENT PRIMARY KEY,
    questionnaire_id INT NOT NULL,
    respondent_id INT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (questionnaire_id) REFERENCES questionnaires(id) ON DELETE CASCADE,
    FOREIGN KEY (respondent_id) REFERENCES users(id) ON DELETE SET NULL
) ENGINE=InnoDB;

-- 创建问题回答表
CREATE TABLE IF NOT EXISTS question_responses (
    id INT AUTO_INCREMENT PRIMARY KEY,
    questionnaire_response_id INT NOT NULL,
    question_id INT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (questionnaire_response_id) REFERENCES questionnaire_responses(id) ON DELETE CASCADE,
    FOREIGN KEY (question_id) REFERENCES questions(id) ON DELETE CASCADE
) ENGINE=InnoDB;

-- 创建文本回答表
CREATE TABLE IF NOT EXISTS text_responses (
    id INT AUTO_INCREMENT PRIMARY KEY,
    question_response_id INT NOT NULL,
    text_value TEXT NOT NULL,
    FOREIGN KEY (question_response_id) REFERENCES question_responses(id) ON DELETE CASCADE
) ENGINE=InnoDB;

-- 创建选项回答表
CREATE TABLE IF NOT EXISTS option_responses (
    id INT AUTO_INCREMENT PRIMARY KEY,
    question_response_id INT NOT NULL,
    option_id INT NOT NULL,
    FOREIGN KEY (question_response_id) REFERENCES question_responses(id) ON DELETE CASCADE,
    FOREIGN KEY (option_id) REFERENCES question_options(id) ON DELETE CASCADE
) ENGINE=InnoDB; 
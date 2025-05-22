CREATE DATABASE IF NOT EXISTS axelte;
USE axelte;

CREATE TABLE users (
    id CHAR(36) PRIMARY KEY,                          
    username VARCHAR(100) NOT NULL UNIQUE,            
    email VARCHAR(255) NOT NULL UNIQUE,                
    password VARCHAR(255) NOT NULL,   
    avatar VARCHAR(255) DEFAULT 'https://avatars.githubusercontent.com/u/124599?v=4',
    role VARCHAR(255) DEFAULT 'user',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,    
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE workspaces (
  id CHAR(36) PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  description TEXT NULL,
  owner_id CHAR(36) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE TABLE workspace_members (
  workspace_id CHAR(36) NOT NULL,
  user_id      CHAR(36) NOT NULL,
  role ENUM('owner','editor','viewer') NOT NULL DEFAULT 'viewer',
  joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (workspace_id, user_id),
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
  FOREIGN KEY (user_id)      REFERENCES users(id)      ON DELETE CASCADE
);

CREATE TABLE pages (
  id CHAR(36) PRIMARY KEY,
  workspace_id CHAR(36) NOT NULL,
  title VARCHAR(255) NOT NULL,
  content LONGTEXT NULL,
  created_by CHAR(36) NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE,
  FOREIGN KEY (created_by)   REFERENCES users(id)
);
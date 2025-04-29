USE questionnaire;

-- 插入示例用户（密码均为'password'）
INSERT INTO users (username, nickname, password_hash, email) VALUES
('admin', '管理员', '$2y$12$8Wz1xNV.J9LY4Zh2bT1yS.TKkqf6TUINYwMvWNGBJ9Yf7A32Ejspy', 'admin@example.com'),
('user1', '测试用户1', '$2y$12$8Wz1xNV.J9LY4Zh2bT1yS.TKkqf6TUINYwMvWNGBJ9Yf7A32Ejspy', 'user1@example.com'),
('user2', '测试用户2', '$2y$12$8Wz1xNV.J9LY4Zh2bT1yS.TKkqf6TUINYwMvWNGBJ9Yf7A32Ejspy', 'user2@example.com');

-- 插入示例问卷
INSERT INTO questionnaires (title, description, is_public, creator_id) VALUES
('产品满意度调查', '这是一份用于调查用户对我们产品满意度的问卷', TRUE, 1),
('用户体验调查', '了解用户使用我们产品时的体验', TRUE, 2),
('学习情况反馈', '了解学生对课程的学习情况', FALSE, 1);

-- 插入示例问题
-- 产品满意度调查问题
INSERT INTO questions (questionnaire_id, title, question_type, required, display_order) VALUES
(1, '您对我们的产品总体满意度如何？', 'radio', TRUE, 1),
(1, '您使用我们产品的频率是？', 'radio', TRUE, 2),
(1, '您最喜欢产品的哪些功能？', 'checkbox', TRUE, 3),
(1, '您对产品有什么建议？', 'text', FALSE, 4);

-- 用户体验调查问题
INSERT INTO questions (questionnaire_id, title, question_type, required, display_order) VALUES
(2, '您觉得产品的界面设计如何？', 'radio', TRUE, 1),
(2, '您在使用过程中遇到了哪些问题？', 'checkbox', FALSE, 2),
(2, '您希望我们改进哪些方面？', 'text', FALSE, 3);

-- 学习情况反馈问题
INSERT INTO questions (questionnaire_id, title, question_type, required, display_order) VALUES
(3, '您对本课程的内容理解程度如何？', 'radio', TRUE, 1),
(3, '您对课程哪些部分感兴趣？', 'checkbox', TRUE, 2),
(3, '您希望课程增加哪些内容？', 'text', FALSE, 3);

-- 插入示例问题选项
-- 产品满意度调查问题选项
INSERT INTO question_options (question_id, option_text, display_order) VALUES
(1, '非常满意', 1),
(1, '满意', 2),
(1, '一般', 3),
(1, '不满意', 4),
(1, '非常不满意', 5),
(2, '每天', 1),
(2, '每周几次', 2),
(2, '每月几次', 3),
(2, '很少使用', 4),
(3, '界面设计', 1),
(3, '功能丰富', 2),
(3, '使用简便', 3),
(3, '性能稳定', 4),
(3, '客户服务', 5);

-- 用户体验调查问题选项
INSERT INTO question_options (question_id, option_text, display_order) VALUES
(5, '非常好', 1),
(5, '良好', 2),
(5, '一般', 3),
(5, '较差', 4),
(5, '非常差', 5),
(6, '加载速度慢', 1),
(6, '功能不易找到', 2),
(6, '界面复杂', 3),
(6, '操作不便', 4),
(6, '崩溃或错误', 5);

-- 学习情况反馈问题选项
INSERT INTO question_options (question_id, option_text, display_order) VALUES
(8, '完全理解', 1),
(8, '大部分理解', 2),
(8, '部分理解', 3),
(8, '很少理解', 4),
(8, '完全不理解', 5),
(9, '理论知识', 1),
(9, '实践案例', 2),
(9, '课堂讨论', 3),
(9, '项目实践', 4),
(9, '小组协作', 5);

-- 插入示例问卷响应
INSERT INTO questionnaire_responses (questionnaire_id, respondent_id, created_at) VALUES
(1, 2, NOW() - INTERVAL 2 DAY),
(1, 3, NOW() - INTERVAL 1 DAY),
(1, NULL, NOW());

-- 插入示例问题响应
-- 用户2对产品满意度调查的回答
INSERT INTO question_responses (questionnaire_response_id, question_id, created_at) VALUES
(1, 1, NOW() - INTERVAL 2 DAY),
(1, 2, NOW() - INTERVAL 2 DAY),
(1, 3, NOW() - INTERVAL 2 DAY),
(1, 4, NOW() - INTERVAL 2 DAY);

-- 用户3对产品满意度调查的回答
INSERT INTO question_responses (questionnaire_response_id, question_id, created_at) VALUES
(2, 1, NOW() - INTERVAL 1 DAY),
(2, 2, NOW() - INTERVAL 1 DAY),
(2, 3, NOW() - INTERVAL 1 DAY),
(2, 4, NOW() - INTERVAL 1 DAY);

-- 匿名用户对产品满意度调查的回答
INSERT INTO question_responses (questionnaire_response_id, question_id, created_at) VALUES
(3, 1, NOW()),
(3, 2, NOW()),
(3, 3, NOW()),
(3, 4, NOW());

-- 插入示例选项响应
-- 用户2的单选和多选回答
INSERT INTO option_responses (question_response_id, option_id) VALUES
(1, 2), -- 满意
(2, 1), -- 每天
(3, 10), -- 界面设计
(3, 12); -- 使用简便

-- 用户3的单选和多选回答
INSERT INTO option_responses (question_response_id, option_id) VALUES
(5, 1), -- 非常满意
(6, 2), -- 每周几次
(7, 11), -- 功能丰富
(7, 13); -- 性能稳定

-- 匿名用户的单选和多选回答
INSERT INTO option_responses (question_response_id, option_id) VALUES
(9, 3), -- 一般
(10, 3), -- 每月几次
(11, 10), -- 界面设计
(11, 14); -- 客户服务

-- 插入示例文本响应
-- 用户2的文本回答
INSERT INTO text_responses (question_response_id, text_value) VALUES
(4, '希望能增加更多个性化设置选项，并优化移动端体验。');

-- 用户3的文本回答
INSERT INTO text_responses (question_response_id, text_value) VALUES
(8, '建议增加数据导出功能，方便用户备份和分析。');

-- 匿名用户的文本回答
INSERT INTO text_responses (question_response_id, text_value) VALUES
(12, '界面可以再简洁一些，功能层级太多，不容易找到想要的选项。'); 
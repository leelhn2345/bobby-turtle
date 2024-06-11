alter table telegram_whisperers
add constraint telegram_whisperers_telegram_chat_id_fkey foreign key (telegram_chat_id) references chatrooms (id);

-- Create notifications table
CREATE TABLE notifications (
    id UUID PRIMARY KEY,
    message_id UUID,
    friend_request_id UUID,
    user_id UUID NOT NULL,
    channel_id UUID NOT NULL,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    notification_type TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at TIMESTAMPTZ,
    read_at TIMESTAMPTZ,
    metadata JSONB
);

-- Create indexes for notifications table
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_channel_id ON notifications(channel_id);
CREATE INDEX idx_notifications_status ON notifications(status);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);

-- Create notification_preferences table
CREATE TABLE notification_preferences (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    channel_id UUID NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    muted_until TIMESTAMPTZ,
    UNIQUE(user_id, channel_id)
);

-- Create indexes for notification_preferences table
CREATE INDEX idx_notification_preferences_user_id ON notification_preferences(user_id);
CREATE INDEX idx_notification_preferences_channel_id ON notification_preferences(channel_id);
CREATE INDEX idx_notification_preferences_enabled ON notification_preferences(enabled);
CREATE INDEX idx_notification_preferences_muted_until ON notification_preferences(muted_until);

use crate::config::AppConfig;
use crate::error::AppError;
use lettre::message::MultiPart;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::sync::Arc;
use tokio::sync::RwLock;

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[derive(Debug, Clone)]
pub struct SentEmail {
    pub to: String,
    pub subject: String,
    pub text_body: String,
    pub html_body: String,
}

pub struct EmailService {
    config: Arc<AppConfig>,
    captured_emails: Arc<RwLock<Vec<SentEmail>>>,
}

impl EmailService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self {
            config,
            captured_emails: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn captured_emails(&self) -> Vec<SentEmail> {
        self.captured_emails.read().await.clone()
    }

    pub async fn clear_captured_emails(&self) {
        self.captured_emails.write().await.clear();
    }

    pub async fn send_magic_link(
        &self,
        to_email: &str,
        user_name: &str,
        magic_token: &str,
        locale: &str,
    ) -> Result<(), AppError> {
        let tree_title = &self.config.tree_title;
        let origin = self
            .config
            .app_origin
            .split([',', ';'])
            .map(|s| s.trim().trim_end_matches('/'))
            .find(|s| !s.is_empty())
            .unwrap_or("http://localhost:8080");
        let magic_link_url = format!("{origin}/magic-link/verify?token={magic_token}");

        let safe_title = html_escape(tree_title);
        let (subject, text_body, html_body) = if locale.starts_with("ru") {
            let name_display = if user_name.trim().is_empty() {
                "Пользователь"
            } else {
                user_name.trim()
            };
            let safe_name = html_escape(name_display);
            let subject = format!("Вход в {tree_title}");
            let text = format!(
                "Здравствуйте, {name_display}!\n\n\
                 Нажмите на ссылку ниже для входа в {tree_title}. Ссылка действительна в течение 15 минут и может быть использована только один раз.\n\n\
                 Войти в семейный архив: {magic_link_url}\n\n\
                 Если вы не запрашивали это письмо, просто проигнорируйте его. Ваша учётная запись в безопасности.\n"
            );
            let html = format!(
                r#"<!DOCTYPE html>
<html lang="ru">
<head>
  <meta charset="utf-8">
  <title>Вход в {safe_title}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #1f2937; max-width: 600px; margin: 0 auto; padding: 24px;">
  <h2 style="color: #111827; margin-bottom: 16px;">Вход в {safe_title}</h2>
  <p>Здравствуйте, {safe_name}!</p>
  <p>Нажмите кнопку ниже для входа в <strong>{safe_title}</strong>. Ссылка действительна в течение <strong>15 минут</strong> и может быть использована только один раз.</p>
  <p style="margin: 28px 0;">
    <a href="{magic_link_url}" style="background-color: #2563eb; color: #ffffff; padding: 12px 24px; text-decoration: none; border-radius: 6px; font-weight: 600; display: inline-block;">Войти в семейный архив</a>
  </p>
  <p style="font-size: 14px; color: #4b5563;">Или скопируйте эту ссылку в адресную строку браузера:<br><a href="{magic_link_url}" style="color: #2563eb; word-break: break-all;">{magic_link_url}</a></p>
  <hr style="border: none; border-top: 1px solid #e5e7eb; margin: 28px 0;" />
  <p style="font-size: 12px; color: #6b7280;">Если вы не запрашивали это письмо, просто проигнорируйте его. Ваша учётная запись в безопасности.</p>
</body>
</html>"#
            );
            (subject, text, html)
        } else {
            let name_display = if user_name.trim().is_empty() {
                "there"
            } else {
                user_name.trim()
            };
            let safe_name = html_escape(name_display);
            let subject = format!("Sign in to {tree_title}");
            let text = format!(
                "Hello {name_display},\n\n\
                 Click the link below to sign in to {tree_title}. This link is valid for 15 minutes and can only be used once.\n\n\
                 Sign In to Family Archive: {magic_link_url}\n\n\
                 If you did not request this link, you can safely ignore this email. Your account remains secure.\n"
            );
            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Sign in to {safe_title}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #1f2937; max-width: 600px; margin: 0 auto; padding: 24px;">
  <h2 style="color: #111827; margin-bottom: 16px;">Sign in to {safe_title}</h2>
  <p>Hello {safe_name},</p>
  <p>Click the button below to sign in to <strong>{safe_title}</strong>. This link is valid for <strong>15 minutes</strong> and can only be used once.</p>
  <p style="margin: 28px 0;">
    <a href="{magic_link_url}" style="background-color: #2563eb; color: #ffffff; padding: 12px 24px; text-decoration: none; border-radius: 6px; font-weight: 600; display: inline-block;">Sign In to Family Archive</a>
  </p>
  <p style="font-size: 14px; color: #4b5563;">Or copy and paste this URL into your browser:<br><a href="{magic_link_url}" style="color: #2563eb; word-break: break-all;">{magic_link_url}</a></p>
  <hr style="border: none; border-top: 1px solid #e5e7eb; margin: 28px 0;" />
  <p style="font-size: 12px; color: #6b7280;">If you did not request this link, you can safely ignore this email. Your account remains secure.</p>
</body>
</html>"#
            );
            (subject, text, html)
        };

        self.dispatch_email(to_email, subject, text_body, html_body)
            .await
    }

    pub async fn send_invitation(
        &self,
        to_email: &str,
        _invitee_name: &str,
        inviter_name: &str,
        role: &str,
        invite_token: &str,
        locale: &str,
    ) -> Result<(), AppError> {
        let tree_title = &self.config.tree_title;
        let origin = self
            .config
            .app_origin
            .split([',', ';'])
            .map(|s| s.trim().trim_end_matches('/'))
            .find(|s| !s.is_empty())
            .unwrap_or("http://localhost:8080");
        let invite_url = format!("{origin}/register?invite_token={invite_token}");

        let inviter_display = if inviter_name.trim().is_empty() {
            "A family member"
        } else {
            inviter_name.trim()
        };

        let safe_title = html_escape(tree_title);
        let safe_role = html_escape(role);
        let (subject, text_body, html_body) = if locale.starts_with("ru") {
            let inviter_ru = if inviter_name.trim().is_empty() {
                "Участник семьи"
            } else {
                inviter_name.trim()
            };
            let safe_inviter_ru = html_escape(inviter_ru);
            let subject = format!("Приглашение в {tree_title}");
            let text = format!(
                "Здравствуйте!\n\n\
                 {inviter_ru} приглашает вас исследовать и сохранять семейную историю в архиве {tree_title} с правами {role}.\n\n\
                 Принять приглашение и войти: {invite_url}\n\n\
                 Ссылка-приглашение действительна в течение 7 дней.\n"
            );
            let html = format!(
                r#"<!DOCTYPE html>
<html lang="ru">
<head>
  <meta charset="utf-8">
  <title>Приглашение в {safe_title}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #1f2937; max-width: 600px; margin: 0 auto; padding: 24px;">
  <h2 style="color: #111827; margin-bottom: 16px;">Приглашение в {safe_title}</h2>
  <p>Здравствуйте!</p>
  <p><strong>{safe_inviter_ru}</strong> приглашает вас исследовать и сохранять семейную историю в архиве <strong>{safe_title}</strong> с правами <strong>{safe_role}</strong>.</p>
  <p style="margin: 28px 0;">
    <a href="{invite_url}" style="background-color: #2563eb; color: #ffffff; padding: 12px 24px; text-decoration: none; border-radius: 6px; font-weight: 600; display: inline-block;">Принять приглашение и войти</a>
  </p>
  <p style="font-size: 14px; color: #4b5563;">Или скопируйте эту ссылку в адресную строку браузера:<br><a href="{invite_url}" style="color: #2563eb; word-break: break-all;">{invite_url}</a></p>
  <hr style="border: none; border-top: 1px solid #e5e7eb; margin: 28px 0;" />
  <p style="font-size: 12px; color: #6b7280;">Ссылка-приглашение действительна в течение 7 дней.</p>
</body>
</html>"#
            );
            (subject, text, html)
        } else {
            let safe_inviter_en = html_escape(inviter_display);
            let subject = format!("Invitation to join {tree_title}");
            let text = format!(
                "Hello,\n\n\
                 {inviter_display} has invited you to view and preserve family history in {tree_title} as an {role}.\n\n\
                 Accept Invitation & Create Account: {invite_url}\n\n\
                 This invitation link is valid for 7 days.\n"
            );
            let html = format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Invitation to join {safe_title}</title>
</head>
<body style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #1f2937; max-width: 600px; margin: 0 auto; padding: 24px;">
  <h2 style="color: #111827; margin-bottom: 16px;">Invitation to join {safe_title}</h2>
  <p>Hello,</p>
  <p><strong>{safe_inviter_en}</strong> has invited you to view and preserve family history in <strong>{safe_title}</strong> as an <strong>{safe_role}</strong>.</p>
  <p style="margin: 28px 0;">
    <a href="{invite_url}" style="background-color: #2563eb; color: #ffffff; padding: 12px 24px; text-decoration: none; border-radius: 6px; font-weight: 600; display: inline-block;">Accept Invitation & Create Account</a>
  </p>
  <p style="font-size: 14px; color: #4b5563;">Or copy and paste this URL into your browser:<br><a href="{invite_url}" style="color: #2563eb; word-break: break-all;">{invite_url}</a></p>
  <hr style="border: none; border-top: 1px solid #e5e7eb; margin: 28px 0;" />
  <p style="font-size: 12px; color: #6b7280;">This invitation link is valid for 7 days.</p>
</body>
</html>"#
            );
            (subject, text, html)
        };

        self.dispatch_email(to_email, subject, text_body, html_body)
            .await
    }

    async fn dispatch_email(
        &self,
        to_email: &str,
        subject: String,
        text_body: String,
        html_body: String,
    ) -> Result<(), AppError> {
        let sent = SentEmail {
            to: to_email.to_string(),
            subject: subject.clone(),
            text_body: text_body.clone(),
            html_body: html_body.clone(),
        };

        // If test environment or smtp_host is empty, capture email in memory without network call
        if self.config.app_env == "test" || self.config.smtp_host.is_empty() {
            tracing::info!(to = %to_email, subject = %subject, "Captured transactional email in memory (test/mock mode)");
            self.captured_emails.write().await.push(sent);
            return Ok(());
        }

        // Build Lettre Message
        let from_addr = if self.config.smtp_from_email.is_empty() {
            "noreply@familytree.local"
        } else {
            &self.config.smtp_from_email
        };
        let from_header = format!("\"{}\" <{}>", self.config.smtp_from_name, from_addr);
        let from_mailbox: lettre::message::Mailbox = from_header
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid SMTP from address: {e}")))?;

        let to_mailbox: lettre::message::Mailbox = to_email
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid recipient email address: {e}")))?;

        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .multipart(MultiPart::alternative_plain_html(text_body, html_body))
            .map_err(|e| AppError::Internal(format!("Failed to build email message: {e}")))?;

        // Build Lettre transport
        let mut transport_builder =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)
                .map_err(|e| AppError::Internal(format!("Failed to resolve SMTP host: {e}")))?
                .port(self.config.smtp_port);

        if !self.config.smtp_user.is_empty() || !self.config.smtp_password.is_empty() {
            let creds = Credentials::new(
                self.config.smtp_user.clone(),
                self.config.smtp_password.clone(),
            );
            transport_builder = transport_builder.credentials(creds);
        }

        let transport = transport_builder.build();

        tracing::info!(to = %to_email, "Dispatching transactional email via SMTP");
        transport
            .send(email)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send transactional email: {e}")))?;

        Ok(())
    }
}

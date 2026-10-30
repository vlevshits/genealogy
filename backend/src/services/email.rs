use crate::config::AppConfig;
use crate::error::AppError;
use lettre::message::MultiPart;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn html_escape(s: &str) -> String {
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

pub struct EmailLayout<'a> {
    pub lang: &'a str,
    pub page_title: &'a str,
    pub preheader: &'a str,
    pub tree_title: &'a str,
    pub subtitle: &'a str,
    pub content_html: &'a str,
    pub footer_notice: &'a str,
}

pub fn render_button(url: &str, label: &str) -> String {
    let safe_url = html_escape(url);
    let safe_label = html_escape(label);
    format!(
        r##"<table role="presentation" border="0" cellpadding="0" cellspacing="0" style="margin: 28px 0;">
  <tr>
    <td align="center" bgcolor="#2D5584" style="background-color: #2D5584; border-radius: 8px; text-align: center; mso-padding-alt: 14px 28px;">
      <!--[if mso]>
      <a href="{safe_url}" style="color: #FFFFFF; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 15px; font-weight: bold; text-decoration: none; display: inline-block; padding: 14px 28px; background-color: #2D5584; border-radius: 8px;">
        <span style="color: #FFFFFF; font-weight: bold;">{safe_label}</span>
      </a>
      <![endif]-->
      <!--[if !mso]><!-->
      <a href="{safe_url}" target="_blank" style="display: inline-block; padding: 14px 28px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 15px; font-weight: 700; color: #FFFFFF; text-decoration: none; border-radius: 8px; background-color: #2D5584; border: 1px solid #2D5584; text-align: center;">
        <span style="color: #FFFFFF; text-decoration: none;">{safe_label}</span>
      </a>
      <!--<![endif]-->
    </td>
  </tr>
</table>"##
    )
}

pub fn render_url_fallback(label: &str, url: &str) -> String {
    let safe_label = html_escape(label);
    let safe_url = html_escape(url);
    format!(
        r##"<table role="presentation" border="0" cellpadding="0" cellspacing="0" width="100%" class="email-url-fallback" style="background-color: #F8FAFC; border: 1px solid #E2E8F0; border-radius: 8px; margin-top: 24px;">
  <tr>
    <td style="padding: 14px 16px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
      <p class="email-fallback-label" style="margin: 0 0 6px 0; font-size: 13px; line-height: 1.4; color: #64748B;">
        {safe_label}
      </p>
      <div style="font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; font-size: 12px; line-height: 1.5; word-break: break-all;">
        <a href="{safe_url}" class="email-fallback-link" style="color: #2D5584; text-decoration: underline; word-break: break-all;">{safe_url}</a>
      </div>
    </td>
  </tr>
</table>"##
    )
}

pub fn render_invitation_callout(
    role_label: &str,
    role_display: &str,
    validity_label: &str,
    validity_value: &str,
) -> String {
    let safe_role_label = html_escape(role_label);
    let safe_role_display = html_escape(role_display);
    let safe_validity_label = html_escape(validity_label);
    let safe_validity_value = html_escape(validity_value);
    format!(
        r##"<table role="presentation" border="0" cellpadding="0" cellspacing="0" width="100%" class="email-callout" style="background-color: #F8FAFC; border: 1px solid #E2E8F0; border-left: 3px solid #C68A34; border-radius: 6px; margin: 20px 0;">
  <tr>
    <td style="padding: 14px 16px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
      <div style="font-size: 13px; color: #475569; margin-bottom: 6px;">
        <span style="font-weight: 600; color: #1E293B; margin-right: 8px;">{safe_role_label}</span>
        <span class="email-role-pill" style="display: inline-block; background-color: #FEF3C7; color: #92400E; border: 1px solid #FDE68A; font-size: 12px; font-weight: 600; padding: 2px 10px; border-radius: 9999px;">{safe_role_display}</span>
      </div>
      <div class="email-validity-note" style="font-size: 12px; color: #64748B;">
        {safe_validity_label} <strong style="color: #334155;">{safe_validity_value}</strong>
      </div>
    </td>
  </tr>
</table>"##
    )
}

pub fn render_email_shell(layout: EmailLayout) -> String {
    let safe_page_title = html_escape(layout.page_title);
    let safe_preheader = html_escape(layout.preheader);
    let safe_tree_title = html_escape(layout.tree_title);
    let safe_subtitle = html_escape(layout.subtitle);
    let safe_footer_notice = html_escape(layout.footer_notice);

    format!(
        r##"<!DOCTYPE html>
<html lang="{lang}" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <meta http-equiv="X-UA-Compatible" content="IE=edge">
  <meta name="color-scheme" content="light dark">
  <meta name="supported-color-schemes" content="light dark">
  <title>{safe_page_title}</title>
  <!--[if mso]>
  <noscript>
    <xml>
      <o:OfficeDocumentSettings>
        <o:PixelsPerInch>96</o:PixelsPerInch>
      </o:OfficeDocumentSettings>
    </xml>
  </noscript>
  <![endif]-->
  <style>
    :root {{
      color-scheme: light dark;
      supported-color-schemes: light dark;
    }}
    body, table, td, p, a, li, blockquote {{
      -webkit-text-size-adjust: 100%;
      -ms-text-size-adjust: 100%;
    }}
    table, td {{
      mso-table-lspace: 0pt;
      mso-table-rspace: 0pt;
    }}
    /* Gmail web & mobile strips <svg>; reveal fallback tree symbol via Gmail-specific selector */
    u + .email-body .email-emblem-fallback {{
      display: inline-block !important;
      line-height: 48px !important;
    }}
    @media only screen and (max-width: 600px) {{
      .email-container {{
        width: 100% !important;
        max-width: 100% !important;
      }}
      .email-card-body {{
        padding: 24px 20px !important;
      }}
    }}
    @media (prefers-color-scheme: dark) {{
      body, .email-canvas {{
        background-color: #0F172A !important;
      }}
      .email-card {{
        background-color: #1E293B !important;
        border-color: #334155 !important;
      }}
      .email-heading {{
        color: #F8FAFC !important;
      }}
      .email-subheading {{
        color: #94A3B8 !important;
      }}
      .email-greeting, .email-message, .email-text {{
        color: #CBD5E1 !important;
      }}
      .email-security-note {{
        color: #94A3B8 !important;
      }}
      .email-callout {{
        background-color: #0F172A !important;
        border-color: #334155 !important;
        border-left-color: #C68A34 !important;
      }}
      .email-role-pill {{
        background-color: #78350F !important;
        color: #FDE68A !important;
        border-color: #92400E !important;
      }}
      .email-url-fallback {{
        background-color: #0F172A !important;
        border-color: #334155 !important;
      }}
      .email-fallback-label {{
        color: #94A3B8 !important;
      }}
      .email-fallback-link {{
        color: #60A5FA !important;
      }}
      .email-footer-text {{
        color: #64748B !important;
      }}
    }}
  </style>
</head>
<body class="email-body" style="margin: 0; padding: 0; width: 100% !important; background-color: #F8F9FA; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; -webkit-font-smoothing: antialiased; word-spacing: normal;">
  <!-- Hidden Preheader -->
  <div style="display:none;font-size:1px;line-height:1px;max-height:0px;max-width:0px;opacity:0;overflow:hidden;mso-hide:all;font-family:sans-serif;">
    {safe_preheader}
    &#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;&#847;&zwnj;&nbsp;
  </div>

  <!-- Outer Canvas Table -->
  <table role="presentation" border="0" cellpadding="0" cellspacing="0" width="100%" class="email-canvas" style="background-color: #F8F9FA; width: 100% !important; min-width: 100%; margin: 0; padding: 0;">
    <tr>
      <td align="center" style="padding: 40px 16px;">
        <!--[if (gte mso 9)|(IE)]>
        <table role="presentation" align="center" border="0" cellspacing="0" cellpadding="0" width="580" style="width: 580px;">
          <tr>
            <td align="center" valign="top">
        <![endif]-->
        <div class="email-container" style="max-width: 580px; width: 100%; margin: 0 auto; text-align: left;">
          <!-- Card Table -->
          <table role="presentation" border="0" cellpadding="0" cellspacing="0" width="100%" class="email-card" style="background-color: #FFFFFF; border: 1px solid #E2E8F0; border-top: 4px solid #2D5584; border-radius: 12px; border-collapse: separate; overflow: hidden;">
            <tr>
              <td class="email-card-body" style="padding: 36px 32px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;">
                <!-- Brand Emblem Tile (48x48 rounded) -->
                <table role="presentation" border="0" cellpadding="0" cellspacing="0" style="margin-bottom: 20px;">
                  <tr>
                    <td width="48" height="48" bgcolor="#2D5584" align="center" valign="middle" style="width: 48px; height: 48px; background-color: #2D5584; border-radius: 12px; text-align: center; vertical-align: middle; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 24px; line-height: 48px; color: #FFFFFF;">
                      <!--[if mso]>
                      <table role="presentation" border="0" cellspacing="0" cellpadding="0" width="48" height="48">
                        <tr>
                          <td align="center" valign="middle" style="font-family: Arial, sans-serif; font-size: 24px; line-height: 24px; color: #FFFFFF;">
                            &#127795;
                          </td>
                        </tr>
                      </table>
                      <![endif]-->
                      <!--[if !mso]><!-->
                      <span class="email-emblem-fallback" style="display: none; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 24px; line-height: 48px; color: #FFFFFF; text-decoration: none;">&#127795;</span>
                      <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="#FFFFFF" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" style="display: block; margin: 0 auto;">
                        <path d="M12 3a4.5 4.5 0 0 0-4.1 2.65A4 4 0 0 0 5 9.5c0 2.05 1.54 3.74 3.53 3.97h6.94A4 4 0 0 0 19 9.5a4 4 0 0 0-2.9-3.85A4.5 4.5 0 0 0 12 3Z" />
                        <path d="M12 11v9" />
                        <path d="M12 14.5 9.2 11.8" />
                        <path d="M12 13.5 14.8 11" />
                        <path d="M8.5 21c1.1-.8 2.2-1 3.5-1s2.4.2 3.5 1" />
                        <circle cx="12" cy="7.2" r="0.9" fill="#C68A34" stroke="none" />
                        <circle cx="9.2" cy="9.8" r="0.85" fill="#C68A34" stroke="none" />
                        <circle cx="14.8" cy="9.4" r="0.85" fill="#C68A34" stroke="none" />
                      </svg>
                      <!--<![endif]-->
                    </td>
                  </tr>
                </table>

                <!-- Header: Title and Localized Subtitle -->
                <h1 class="email-heading" style="margin: 0 0 4px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 20px; font-weight: 700; line-height: 1.3; color: #1E293B;">
                  {safe_tree_title}
                </h1>
                <p class="email-subheading" style="margin: 0 0 24px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 13px; line-height: 1.4; color: #64748B;">
                  {safe_subtitle}
                </p>

                <!-- Body Content -->
                {content_html}
              </td>
            </tr>
          </table>

          <!-- Centered Footer -->
          <table role="presentation" border="0" cellpadding="0" cellspacing="0" width="100%" style="margin-top: 24px;">
            <tr>
              <td align="center" style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 12px; line-height: 1.5; color: #94A3B8; text-align: center;">
                <p class="email-footer-text" style="margin: 0 0 4px 0; color: #64748B; font-weight: 500;">
                  {safe_tree_title}
                </p>
                <p class="email-footer-text" style="margin: 0; color: #94A3B8;">
                  {safe_footer_notice}
                </p>
              </td>
            </tr>
          </table>
        </div>
        <!--[if (gte mso 9)|(IE)]>
            </td>
          </tr>
        </table>
        <![endif]-->
      </td>
    </tr>
  </table>
</body>
</html>"##,
        lang = layout.lang,
        safe_page_title = safe_page_title,
        safe_preheader = safe_preheader,
        safe_tree_title = safe_tree_title,
        safe_subtitle = safe_subtitle,
        content_html = layout.content_html,
        safe_footer_notice = safe_footer_notice,
    )
}

pub fn build_magic_link_email(
    tree_title: &str,
    user_name: &str,
    magic_link_url: &str,
    locale: &str,
) -> (String, String, String) {
    let safe_title = html_escape(tree_title);
    if locale.starts_with("ru") {
        let name_display = if user_name.trim().is_empty() {
            "Пользователь"
        } else {
            user_name.trim()
        };
        let safe_name = html_escape(name_display);
        let subject = format!("Вход в {tree_title}");
        let text_body = format!(
            "Здравствуйте, {name_display}!\n\n\
             Нажмите на ссылку ниже для входа в {tree_title}. Ссылка действительна в течение 15 минут и может быть использована только один раз.\n\n\
             Войти в семейный архив: {magic_link_url}\n\n\
             Если вы не запрашивали это письмо, просто проигнорируйте его. Ваша учётная запись в безопасности.\n"
        );
        let page_title = format!("Вход в {tree_title}");
        let preheader = format!("Одноразовая ссылка для входа в {tree_title}");
        let subtitle = "Семейный архив и генеалогическое древо";
        let button_html = render_button(magic_link_url, "Войти в семейный архив");
        let fallback_html = render_url_fallback(
            "Или скопируйте эту ссылку в адресную строку браузера:",
            magic_link_url,
        );
        let content_html = format!(
            r##"<p class="email-greeting" style="margin: 0 0 16px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 15px; line-height: 1.6; color: #334155;">Здравствуйте, <strong>{safe_name}</strong>!</p>
<p class="email-message" style="margin: 0 0 20px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 15px; line-height: 1.6; color: #334155;">Нажмите кнопку ниже для входа в <strong>{safe_title}</strong>. Ссылка действительна в течение <strong>15 минут</strong> и может быть использована только один раз.</p>
{button_html}
{fallback_html}
<hr style="border: none; border-top: 1px solid #E2E8F0; margin: 28px 0 20px 0;" />
<p class="email-security-note" style="margin: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 12px; line-height: 1.5; color: #64748B;">Если вы не запрашивали это письмо, просто проигнорируйте его. Ваша учётная запись в безопасности.</p>"##
        );
        let html_body = render_email_shell(EmailLayout {
            lang: "ru",
            page_title: &page_title,
            preheader: &preheader,
            tree_title,
            subtitle,
            content_html: &content_html,
            footer_notice:
                "Это автоматическое служебное уведомление. Пожалуйста, не отвечайте на это письмо.",
        });
        (subject, text_body, html_body)
    } else {
        let name_display = if user_name.trim().is_empty() {
            "there"
        } else {
            user_name.trim()
        };
        let safe_name = html_escape(name_display);
        let subject = format!("Sign in to {tree_title}");
        let text_body = format!(
            "Hello {name_display},\n\n\
             Click the link below to sign in to {tree_title}. This link is valid for 15 minutes and can only be used once.\n\n\
             Sign In to Family Archive: {magic_link_url}\n\n\
             If you did not request this link, you can safely ignore this email. Your account remains secure.\n"
        );
        let page_title = format!("Sign in to {tree_title}");
        let preheader = format!("Your one-time sign-in link for {tree_title}");
        let subtitle = "Family Tree & Heritage Archive";
        let button_html = render_button(magic_link_url, "Sign In to Family Archive");
        let fallback_html = render_url_fallback(
            "Or copy and paste this URL into your browser:",
            magic_link_url,
        );
        let content_html = format!(
            r##"<p class="email-greeting" style="margin: 0 0 16px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 15px; line-height: 1.6; color: #334155;">Hello <strong>{safe_name}</strong>,</p>
<p class="email-message" style="margin: 0 0 20px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 15px; line-height: 1.6; color: #334155;">Click the button below to sign in to <strong>{safe_title}</strong>. This link is valid for <strong>15 minutes</strong> and can only be used once.</p>
{button_html}
{fallback_html}
<hr style="border: none; border-top: 1px solid #E2E8F0; margin: 28px 0 20px 0;" />
<p class="email-security-note" style="margin: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 12px; line-height: 1.5; color: #64748B;">If you did not request this link, you can safely ignore this email. Your account remains secure.</p>"##
        );
        let html_body = render_email_shell(EmailLayout {
            lang: "en",
            page_title: &page_title,
            preheader: &preheader,
            tree_title,
            subtitle,
            content_html: &content_html,
            footer_notice:
                "This is an automated notification. Please do not reply directly to this email.",
        });
        (subject, text_body, html_body)
    }
}

pub fn build_invitation_email(
    tree_title: &str,
    inviter_name: &str,
    role: &str,
    invite_url: &str,
    locale: &str,
) -> (String, String, String) {
    let safe_title = html_escape(tree_title);
    let safe_role = html_escape(role);
    if locale.starts_with("ru") {
        let inviter_ru = if inviter_name.trim().is_empty() {
            "Участник семьи"
        } else {
            inviter_name.trim()
        };
        let safe_inviter = html_escape(inviter_ru);
        let subject = format!("Приглашение в {tree_title}");
        let text_body = format!(
            "Здравствуйте!\n\n\
             {inviter_ru} приглашает вас исследовать и сохранять семейную историю в архиве {tree_title} с правами {role}.\n\n\
             Принять приглашение и войти: {invite_url}\n\n\
             Ссылка-приглашение действительна в течение 7 дней.\n"
        );
        let page_title = format!("Приглашение в {tree_title}");
        let preheader = format!("Приглашение присоединиться к семейному архиву {tree_title}");
        let subtitle = "Семейный архив и генеалогическое древо";
        let callout_html =
            render_invitation_callout("Роль в архиве:", role, "Срок действия ссылки:", "7 дней");
        let button_html = render_button(invite_url, "Принять приглашение и войти");
        let fallback_html = render_url_fallback(
            "Или скопируйте эту ссылку в адресную строку браузера:",
            invite_url,
        );
        let content_html = format!(
            r##"<p class="email-greeting" style="margin: 0 0 16px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 15px; line-height: 1.6; color: #334155;">Здравствуйте!</p>
<p class="email-message" style="margin: 0 0 16px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 15px; line-height: 1.6; color: #334155;"><strong>{safe_inviter}</strong> приглашает вас исследовать и сохранять семейную историю в архиве <strong>{safe_title}</strong> с правами <strong>{safe_role}</strong>.</p>
{callout_html}
{button_html}
{fallback_html}
<hr style="border: none; border-top: 1px solid #E2E8F0; margin: 28px 0 20px 0;" />
<p class="email-security-note" style="margin: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 12px; line-height: 1.5; color: #64748B;">Ссылка-приглашение действительна в течение 7 дней.</p>"##
        );
        let html_body = render_email_shell(EmailLayout {
            lang: "ru",
            page_title: &page_title,
            preheader: &preheader,
            tree_title,
            subtitle,
            content_html: &content_html,
            footer_notice:
                "Это автоматическое служебное уведомление. Пожалуйста, не отвечайте на это письмо.",
        });
        (subject, text_body, html_body)
    } else {
        let inviter_display = if inviter_name.trim().is_empty() {
            "A family member"
        } else {
            inviter_name.trim()
        };
        let safe_inviter = html_escape(inviter_display);
        let subject = format!("Invitation to join {tree_title}");
        let text_body = format!(
            "Hello,\n\n\
             {inviter_display} has invited you to view and preserve family history in {tree_title} as an {role}.\n\n\
             Accept Invitation & Create Account: {invite_url}\n\n\
             This invitation link is valid for 7 days.\n"
        );
        let page_title = format!("Invitation to join {tree_title}");
        let preheader = format!("Invitation to join the family archive {tree_title}");
        let subtitle = "Family Tree & Heritage Archive";
        let callout_html =
            render_invitation_callout("Assigned Role:", role, "Invitation valid for:", "7 days");
        let button_html = render_button(invite_url, "Accept Invitation & Create Account");
        let fallback_html =
            render_url_fallback("Or copy and paste this URL into your browser:", invite_url);
        let content_html = format!(
            r##"<p class="email-greeting" style="margin: 0 0 16px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 15px; line-height: 1.6; color: #334155;">Hello,</p>
<p class="email-message" style="margin: 0 0 20px 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 15px; line-height: 1.6; color: #334155;"><strong>{safe_inviter}</strong> has invited you to view and preserve family history in <strong>{safe_title}</strong> as an <strong>{safe_role}</strong>.</p>
{callout_html}
{button_html}
{fallback_html}
<hr style="border: none; border-top: 1px solid #E2E8F0; margin: 28px 0 20px 0;" />
<p class="email-security-note" style="margin: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; font-size: 12px; line-height: 1.5; color: #64748B;">This invitation link is valid for 7 days.</p>"##
        );
        let html_body = render_email_shell(EmailLayout {
            lang: "en",
            page_title: &page_title,
            preheader: &preheader,
            tree_title,
            subtitle,
            content_html: &content_html,
            footer_notice:
                "This is an automated notification. Please do not reply directly to this email.",
        });
        (subject, text_body, html_body)
    }
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

        let (subject, text_body, html_body) =
            build_magic_link_email(tree_title, user_name, &magic_link_url, locale);

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

        let (subject, text_body, html_body) =
            build_invitation_email(tree_title, inviter_name, role, &invite_url, locale);

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

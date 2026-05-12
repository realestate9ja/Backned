// Enhanced email templates with Cloudinary headers
// This module contains the new redesigned email templates

impl MailService {
    pub fn enhanced_welcome_email(&self, to: String, full_name: &str, action_url: &str) -> OutboundEmail {
        let header_url = header_asset_url(HEADER_WELCOME);
        OutboundEmail {
            to,
            subject: "Welcome to VeriNest".to_string(),
            text: format!(
                "Hi {full_name}, welcome to Verinest. Open your dashboard to get started: {action_url}"
            ),
            html: format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ background: #E8E2DA; font-family: 'DM Sans', sans-serif; padding: 40px 16px 80px; }}
  .email-shell {{ max-width: 560px; margin: 0 auto; background: #ffffff; border-radius: 20px; overflow: hidden; box-shadow: 0 8px 40px rgba(0,0,0,0.10); }}
  .email-header {{ position: relative; width: 100%; height: 200px; background-size: cover; background-position: center; background-repeat: no-repeat; }}
  .email-header-overlay {{ position: absolute; inset: 0; background: linear-gradient(to bottom, rgba(0,0,0,0.1) 0%, rgba(0,0,0,0.4) 100%); display: flex; align-items: center; justify-content: center; flex-direction: column; padding: 20px; text-align: center; }}
  .header-logo {{ display: flex; align-items: center; gap: 10px; margin-bottom: 16px; }}
  .logo-icon {{ width: 36px; height: 36px; background: #C4714A; border-radius: 10px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }}
  .logo-name {{ font-family: 'Cormorant Garamond', serif; font-size: 22px; font-weight: 600; color: #FFFFFF; letter-spacing: 0.03em; line-height: 1; }}
  .logo-name em {{ font-style: italic; color: #C4714A; }}
  .header-badge {{ font-size: 9px; letter-spacing: 0.2em; text-transform: uppercase; color: #FFFFFF; background: rgba(196, 113, 74, 0.9); padding: 5px 10px; border-radius: 20px; backdrop-filter: blur(10px); }}
  .email-body {{ padding: 36px 40px; }}
  .greeting {{ font-size: 15px; font-weight: 500; color: #1A1814; margin-bottom: 14px; }}
  .body-text {{ font-size: 13.5px; color: #5A5248; line-height: 1.75; margin-bottom: 20px; }}
  .steps-row {{ display: flex; gap: 0; margin: 24px 0; }}
  .step-col {{ flex: 1; text-align: center; padding: 0 8px; position: relative; }}
  .step-col:not(:last-child)::after {{ content: ''; position: absolute; top: 18px; right: -1px; width: 2px; height: 2px; background: #EDE8E0; box-shadow: 6px 0 0 #EDE8E0, -6px 0 0 #EDE8E0; }}
  .step-num-circle {{ width: 36px; height: 36px; border-radius: 50%; border: 1.5px solid #C4714A; display: flex; align-items: center; justify-content: center; margin: 0 auto 10px; font-family: 'Cormorant Garamond', serif; font-size: 16px; color: #C4714A; font-weight: 600; }}
  .step-col-title {{ font-size: 11px; font-weight: 500; color: #1A1814; margin-bottom: 4px; }}
  .step-col-text {{ font-size: 10px; color: #9A8F84; line-height: 1.5; }}
  .cta-wrap {{ margin: 28px 0; }}
  .cta-btn {{ display: inline-block; background: #C4714A; color: white; text-decoration: none; font-family: 'DM Sans', sans-serif; font-size: 13px; font-weight: 500; letter-spacing: 0.06em; padding: 15px 32px; border-radius: 12px; }}
  .email-divider {{ height: 1px; background: #EDE8E0; margin: 28px 0; }}
  .email-footer {{ background: #161412; padding: 32px 40px; }}
  .footer-logo-row {{ display: flex; align-items: center; gap: 10px; margin-bottom: 16px; }}
  .footer-logo-name {{ font-family: 'Cormorant Garamond', serif; font-size: 18px; font-weight: 600; color: #FAF7F3; }}
  .footer-logo-name em {{ font-style: italic; color: #C4714A; }}
  .footer-text {{ font-size: 11px; color: #6A5F54; line-height: 1.7; margin-bottom: 20px; }}
  .footer-links {{ display: flex; gap: 20px; margin-bottom: 20px; flex-wrap: wrap; }}
  .footer-links a {{ font-size: 10px; color: #8A7F74; text-decoration: none; letter-spacing: 0.08em; }}
  .footer-divider {{ height: 1px; background: #2A2520; margin-bottom: 16px; }}
  .footer-legal {{ font-size: 10px; color: #4A4038; line-height: 1.6; }}
</style>
</head>
<body>
  <div class="email-shell">
    <div class="email-header" style="background-image: url('{header_url}');">
      <div class="email-header-overlay">
        <div class="header-logo">
          <div class="logo-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
            </svg>
          </div>
          <span class="logo-name">Veri<em>nest</em></span>
        </div>
        <span class="header-badge">Welcome</span>
      </div>
    </div>

    <div class="email-body">
      <p class="greeting">Hi {full_name},</p>
      <p class="body-text">You've just joined a platform built for one reason — to make renting in Nigeria less stressful, less risky, and a lot faster. No more scrolling through fake listings or chasing agents who disappear.</p>
      <p class="body-text">Here's how Verinest works for you:</p>

      <div class="steps-row">
        <div class="step-col">
          <div class="step-num-circle">01</div>
          <div class="step-col-title">Post a Need</div>
          <div class="step-col-text">Tell us your budget, location and timeline</div>
        </div>
        <div class="step-col">
          <div class="step-num-circle">02</div>
          <div class="step-col-title">Get Matched</div>
          <div class="step-col-text">Verified agents respond with real options</div>
        </div>
        <div class="step-col">
          <div class="step-num-circle">03</div>
          <div class="step-col-title">Move In</div>
          <div class="step-col-text">Compare, schedule a viewing, proceed</div>
        </div>
      </div>

      <div class="cta-wrap">
        <a href="{action_url}" class="cta-btn">Post Your First Need →</a>
      </div>

      <div class="email-divider"></div>

      <p class="body-text" style="font-size:12px; color:#9A8F84;">If you have any questions getting started, reply to this email directly. A real person will respond — not a bot.</p>
    </div>

    <div class="email-footer">
      <div class="footer-logo-row">
        <div class="logo-icon" style="width:28px;height:28px;border-radius:8px;">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
            <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
          </svg>
        </div>
        <span class="footer-logo-name">Veri<em>nest</em></span>
      </div>
      <p class="footer-text">Verinest helps seekers, agents, and landlords connect through verified listings, clearer pricing, and faster rental matching across Nigeria.</p>
      <div class="footer-links">
        <a href="#">Browse Homes</a>
        <a href="#">Post a Need</a>
        <a href="#">For Agents</a>
        <a href="#">Help Centre</a>
        <a href="#">Unsubscribe</a>
      </div>
      <div class="footer-divider"></div>
      <p class="footer-legal">© 2026 Verinest. All rights reserved. · Nigeria<br>You are receiving this because you signed up at verinest.ng</p>
    </div>
  </div>
</body>
</html>"#,
                header_url = header_url,
                full_name = full_name,
                action_url = action_url
            ),
        }
    }

    pub fn enhanced_verification_code_email(&self, to: String, full_name: &str, code: &str) -> OutboundEmail {
        let header_url = header_asset_url(HEADER_SECURITY_DARK);
        OutboundEmail {
            to,
            subject: "Your VeriNest verification code".to_string(),
            text: format!("Hello {full_name}, your VeriNest verification code is {code}."),
            html: format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ background: #E8E2DA; font-family: 'DM Sans', sans-serif; padding: 40px 16px 80px; }}
  .email-shell {{ max-width: 560px; margin: 0 auto; background: #ffffff; border-radius: 20px; overflow: hidden; box-shadow: 0 8px 40px rgba(0,0,0,0.10); }}
  .email-header {{ position: relative; width: 100%; height: 200px; background-size: cover; background-position: center; background-repeat: no-repeat; }}
  .email-header-overlay {{ position: absolute; inset: 0; background: linear-gradient(to bottom, rgba(0,0,0,0.1) 0%, rgba(0,0,0,0.4) 100%); display: flex; align-items: center; justify-content: center; flex-direction: column; padding: 20px; text-align: center; }}
  .header-logo {{ display: flex; align-items: center; gap: 10px; margin-bottom: 16px; }}
  .logo-icon {{ width: 36px; height: 36px; background: #C4714A; border-radius: 10px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }}
  .logo-name {{ font-family: 'Cormorant Garamond', serif; font-size: 22px; font-weight: 600; color: #FFFFFF; letter-spacing: 0.03em; line-height: 1; }}
  .logo-name em {{ font-style: italic; color: #C4714A; }}
  .header-badge {{ font-size: 9px; letter-spacing: 0.2em; text-transform: uppercase; color: #FFFFFF; background: rgba(196, 113, 74, 0.9); padding: 5px 10px; border-radius: 20px; backdrop-filter: blur(10px); }}
  .email-body {{ padding: 36px 40px; }}
  .greeting {{ font-size: 15px; font-weight: 500; color: #1A1814; margin-bottom: 14px; }}
  .body-text {{ font-size: 13.5px; color: #5A5248; line-height: 1.75; margin-bottom: 20px; }}
  .otp-wrap {{ background: #FAF7F3; border-radius: 16px; padding: 28px; text-align: center; margin: 24px 0; border: 1px dashed #E0D8CE; }}
  .otp-label {{ font-size: 9px; letter-spacing: 0.25em; text-transform: uppercase; color: #9A8F84; margin-bottom: 14px; }}
  .otp-code {{ font-family: 'Cormorant Garamond', serif; font-size: 52px; font-weight: 600; color: #C4714A; letter-spacing: 0.18em; line-height: 1; margin-bottom: 10px; }}
  .otp-expiry {{ font-size: 11px; color: #B0A090; }}
  .alert-strip {{ background: #FFF3ED; border-left: 3px solid #C4714A; padding: 14px 18px; border-radius: 0 10px 10px 0; margin-bottom: 20px; }}
  .alert-strip p {{ font-size: 12px; color: #7A4020; line-height: 1.6; }}
  .email-divider {{ height: 1px; background: #EDE8E0; margin: 28px 0; }}
  .email-footer {{ background: #161412; padding: 32px 40px; }}
  .footer-logo-row {{ display: flex; align-items: center; gap: 10px; margin-bottom: 16px; }}
  .footer-logo-name {{ font-family: 'Cormorant Garamond', serif; font-size: 18px; font-weight: 600; color: #FAF7F3; }}
  .footer-logo-name em {{ font-style: italic; color: #C4714A; }}
  .footer-text {{ font-size: 11px; color: #6A5F54; line-height: 1.7; margin-bottom: 20px; }}
  .footer-divider {{ height: 1px; background: #2A2520; margin-bottom: 16px; }}
  .footer-legal {{ font-size: 10px; color: #4A4038; line-height: 1.6; }}
</style>
</head>
<body>
  <div class="email-shell">
    <div class="email-header" style="background-image: url('{header_url}');">
      <div class="email-header-overlay">
        <div class="header-logo">
          <div class="logo-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
            </svg>
          </div>
          <span class="logo-name">Veri<em>nest</em></span>
        </div>
        <span class="header-badge">Security</span>
      </div>
    </div>

    <div class="email-body">
      <p class="greeting">Hi {full_name},</p>
      <p class="body-text">You requested a verification code for your Verinest account. Enter this code in app to continue.</p>

      <div class="otp-wrap">
        <p class="otp-label">Your verification code</p>
        <div class="otp-code">{code}</div>
        <p class="otp-expiry">Expires in 10 minutes · Do not share this code</p>
      </div>

      <div class="alert-strip">
        <p>⚠️ Verinest will never ask for this code by phone or WhatsApp. If someone is asking for it, this is a scam attempt. Do not share it.</p>
      </div>

      <p class="body-text" style="font-size:12px;color:#9A8F84;">If you didn't request this code, you can safely ignore this email. Your account remains secure.</p>
    </div>

    <div class="email-footer">
      <div class="footer-logo-row">
        <div class="logo-icon" style="width:28px;height:28px;border-radius:8px;">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
            <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
          </svg>
        </div>
        <span class="footer-logo-name">Veri<em>nest</em></span>
      </div>
      <p class="footer-text">This is an automated security email from Verinest. If you have concerns about your account security, contact us at security@verinest.ng</p>
      <div class="footer-divider"></div>
      <p class="footer-legal">© 2026 Verinest. All rights reserved.  Nigeria</p>
    </div>
  </div>
</body>
</html>"#,
                header_url = header_url,
                full_name = full_name,
                code = code
            ),
        }
    }

    pub fn enhanced_new_match_email(
        &self,
        to: String,
        full_name: &str,
        property_title: &str,
        property_price: &str,
        agent_name: &str,
        agent_title: &str,
        action_url: &str,
    ) -> OutboundEmail {
        let header_url = header_asset_url(HEADER_NEW_MATCH);
        OutboundEmail {
            to,
            subject: "A verified agent responded to your need".to_string(),
            text: format!(
                "Hi {full_name}, {agent_name} responded to your need for {property_title}. View listing: {action_url}"
            ),
            html: format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ background: #E8E2DA; font-family: 'DM Sans', sans-serif; padding: 40px 16px 80px; }}
  .email-shell {{ max-width: 560px; margin: 0 auto; background: #ffffff; border-radius: 20px; overflow: hidden; box-shadow: 0 8px 40px rgba(0,0,0,0.10); }}
  .email-header {{ position: relative; width: 100%; height: 200px; background-size: cover; background-position: center; background-repeat: no-repeat; }}
  .email-header-overlay {{ position: absolute; inset: 0; background: linear-gradient(to bottom, rgba(0,0,0,0.1) 0%, rgba(0,0,0,0.4) 100%); display: flex; align-items: center; justify-content: center; flex-direction: column; padding: 20px; text-align: center; }}
  .header-logo {{ display: flex; align-items: center; gap: 10px; margin-bottom: 16px; }}
  .logo-icon {{ width: 36px; height: 36px; background: #C4714A; border-radius: 10px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }}
  .logo-name {{ font-family: 'Cormorant Garamond', serif; font-size: 22px; font-weight: 600; color: #FFFFFF; letter-spacing: 0.03em; line-height: 1; }}
  .logo-name em {{ font-style: italic; color: #C4714A; }}
  .header-badge {{ font-size: 9px; letter-spacing: 0.2em; text-transform: uppercase; color: #FFFFFF; background: rgba(196, 113, 74, 0.9); padding: 5px 10px; border-radius: 20px; backdrop-filter: blur(10px); }}
  .email-body {{ padding: 36px 40px; }}
  .greeting {{ font-size: 15px; font-weight: 500; color: #1A1814; margin-bottom: 14px; }}
  .body-text {{ font-size: 13.5px; color: #5A5248; line-height: 1.75; margin-bottom: 20px; }}
  .property-card {{ border: 1px solid #EDE8E0; border-radius: 16px; overflow: hidden; margin-bottom: 16px; }}
  .property-img {{ width: 100%; height: 160px; background: linear-gradient(135deg, #C4714A 0%, #A85D38 40%, #8B4A2A 100%); display: flex; align-items: center; justify-content: center; position: relative; overflow: hidden; }}
  .property-img::before {{ content: ''; position: absolute; inset: 0; background: linear-gradient(to bottom, transparent 40%, rgba(0,0,0,0.5) 100%); }}
  .property-img-label {{ position: absolute; bottom: 14px; left: 16px; right: 16px; z-index: 1; }}
  .property-img-label .p-name {{ font-family: 'Cormorant Garamond', serif; font-size: 20px; font-weight: 600; color: white; line-height: 1; margin-bottom: 3px; }}
  .property-img-label .p-loc {{ font-size: 10px; color: rgba(255,255,255,0.75); letter-spacing: 0.12em; text-transform: uppercase; }}
  .property-verified {{ position: absolute; top: 12px; right: 12px; background: #C4714A; color: white; font-size: 8px; letter-spacing: 0.18em; text-transform: uppercase; padding: 4px 9px; border-radius: 20px; font-weight: 500; z-index: 1; }}
  .property-details {{ padding: 16px 18px; display: flex; align-items: center; justify-content: space-between; }}
  .property-price {{ font-family: 'Cormorant Garamond', serif; font-size: 20px; font-weight: 600; color: #1A1814; }}
  .property-price span {{ font-family: 'DM Sans', sans-serif; font-size: 10px; color: #9A8F84; font-weight: 400; display: block; }}
  .property-meta {{ display: flex; gap: 14px; }}
  .prop-meta-item {{ font-size: 11px; color: #9A8F84; display: flex; align-items: center; gap: 4px; }}
  .agent-card {{ display: flex; align-items: center; gap: 16px; background: #FAF7F3; border-radius: 14px; padding: 18px 20px; margin: 20px 0; border: 1px solid #EDE8E0; }}
  .agent-avatar {{ width: 52px; height: 52px; border-radius: 50%; background: linear-gradient(135deg, #C4714A, #A85D38); display: flex; align-items: center; justify-content: center; flex-shrink: 0; font-family: 'Cormorant Garamond', serif; font-size: 22px; color: white; font-weight: 600; }}
  .agent-info .name {{ font-size: 14px; font-weight: 500; color: #1A1814; margin-bottom: 2px; }}
  .agent-info .role {{ font-size: 11px; color: #9A8F84; margin-bottom: 6px; }}
  .verified-pill {{ display: inline-flex; align-items: center; gap: 4px; background: #EBF7EF; color: #3A8A52; font-size: 9px; font-weight: 500; letter-spacing: 0.15em; text-transform: uppercase; padding: 3px 8px; border-radius: 20px; }}
  .cta-wrap {{ margin: 28px 0; }}
  .cta-btn {{ display: inline-block; background: #C4714A; color: white; text-decoration: none; font-family: 'DM Sans', sans-serif; font-size: 13px; font-weight: 500; letter-spacing: 0.06em; padding: 15px 32px; border-radius: 12px; }}
  .cta-btn.ghost-btn {{ background: transparent; color: #C4714A; border: 1.5px solid #C4714A; }}
  .email-divider {{ height: 1px; background: #EDE8E0; margin: 28px 0; }}
  .email-footer {{ background: #161412; padding: 32px 40px; }}
  .footer-logo-row {{ display: flex; align-items: center; gap: 10px; margin-bottom: 16px; }}
  .footer-logo-name {{ font-family: 'Cormorant Garamond', serif; font-size: 18px; font-weight: 600; color: #FAF7F3; }}
  .footer-logo-name em {{ font-style: italic; color: #C4714A; }}
  .footer-text {{ font-size: 11px; color: #6A5F54; line-height: 1.7; margin-bottom: 20px; }}
  .footer-links {{ display: flex; gap: 20px; margin-bottom: 20px; flex-wrap: wrap; }}
  .footer-links a {{ font-size: 10px; color: #8A7F74; text-decoration: none; letter-spacing: 0.08em; }}
  .footer-divider {{ height: 1px; background: #2A2520; margin-bottom: 16px; }}
  .footer-legal {{ font-size: 10px; color: #4A4038; line-height: 1.6; }}
</style>
</head>
<body>
  <div class="email-shell">
    <div class="email-header" style="background-image: url('{header_url}');">
      <div class="email-header-overlay">
        <div class="header-logo">
          <div class="logo-icon">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none">
              <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
            </svg>
          </div>
          <span class="logo-name">Veri<em>nest</em></span>
        </div>
        <span class="header-badge">New Match</span>
      </div>
    </div>

    <div class="email-body">
      <p class="greeting">Hi {full_name},</p>
      <p class="body-text">Good news — a verified agent responded to your need post for a <strong>3-bedroom apartment in Lekki Phase 1, under ₦3.5M/year.</strong> Here's what they sent:</p>

      <div class="property-card">
        <div class="property-img">
          <div class="property-verified">Verified Listing</div>
          <div class="property-img-label">
            <div class="p-name">{property_title}</div>
            <div class="p-loc">Lekki Phase 1 ·</div>
          </div>
          <svg style="position:absolute;top:50%;left:50%;transform:translate(-50%,-50%);opacity:0.15" width="64" height="64" viewBox="0 0 24 24" fill="none">
            <path d="M3 11L12 3l9 8" stroke="white" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
          </svg>
        </div>
        <div class="property-details">
          <div class="property-price">
            {property_price}
            <span>Per year</span>
          </div>
          <div class="property-meta">
            <div class="prop-meta-item">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none"><path d="M3 12h18M3 6h18M3 18h18" stroke="#9A8F84" stroke-width="2" stroke-linecap="round"/></svg>
              3 bed
            </div>
            <div class="prop-meta-item">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="9" stroke="#9A8F84" stroke-width="2"/></svg>
              2 bath
            </div>
          </div>
        </div>
      </div>

      <div class="agent-card">
        <div class="agent-avatar">{}</div>
        <div class="agent-info">
          <div class="name">{agent_name}</div>
          <div class="role">{agent_title}</div>
          <div class="verified-pill">
            <svg width="8" height="8" viewBox="0 0 12 12" fill="none">
              <circle cx="6" cy="6" r="5" fill="#3A8A52"/>
              <path d="M3.5 6l1.8 1.8 3.2-3.2" stroke="white" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
            </svg>
            Verified Provider
          </div>
        </div>
      </div>

      <div class="cta-wrap" style="display:flex;gap:12px;flex-wrap:wrap;">
        <a href="{action_url}" class="cta-btn">View Full Listing →</a>
        <a href="#" class="cta-btn ghost-btn">Schedule Viewing</a>
      </div>

      <div class="email-divider"></div>
      <p class="body-text" style="font-size:12px;color:#9A8F84;">This agent is verified on Verinest. All communication and viewings should happen through platform — do not send money or sign anything outside of Verinest.</p>
    </div>

    <div class="email-footer">
      <div class="footer-logo-row">
        <div class="logo-icon" style="width:28px;height:28px;border-radius:8px;">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
            <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
            <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
          </svg>
        </div>
        <span class="footer-logo-name">Veri<em>nest</em></span>
      </div>
      <p class="footer-text">Matches expire after 48 hours if not reviewed. Log in to your Verinest dashboard to see all your active matches.</p>
      <div class="footer-links">
        <a href="#">My Dashboard</a>
        <a href="#">My Matches</a>
        <a href="#">Help Centre</a>
        <a href="#">Unsubscribe</a>
      </div>
      <div class="footer-divider"></div>
      <p class="footer-legal">© 2026 Verinest. All rights reserved.  Nigeria</p>
    </div>
  </div>
</body>
</html>"#,
                header_url = header_url,
                full_name = full_name,
                property_title = property_title,
                property_price = property_price,
                agent_name = agent_name,
                agent_title = agent_title,
                action_url = action_url
            ),
        }
    }

    pub fn enhanced_agent_lead_alert(
        &self,
        to: String,
        agent_name: &str,
        seeker_location: &str,
        property_type: &str,
        budget: &str,
        move_in_timeline: &str,
        seeker_name: &str,
        seeker_type: &str,
        additional_notes: &str,
        lead_id: &str,
        action_url: &str,
    ) -> OutboundEmail {
        let header_url = header_asset_url(HEADER_LEAD_ALERT);
        OutboundEmail {
            to,
            subject: format!("🔔 New rental request in {} — {} budget, move-in {}", seeker_location, budget, move_in_timeline),
            text: format!(
                "Hi {agent_name}, A serious seeker just posted a rental request on Verinest that matches your area. Location: {}, Type: {}, Budget: {}, Move-in: {}. Respond now: {action_url}",
                seeker_location, property_type, budget, move_in_timeline
            ),
            html: format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{ background: #E8E2DA; font-family: 'DM Sans', sans-serif; padding: 40px 16px 80px; }}
  .email-shell {{ max-width: 560px; margin: 0 auto; background: #ffffff; border-radius: 0 0 20px 20px; overflow: hidden; box-shadow: 0 8px 40px rgba(0,0,0,0.10); }}
  .subject-line {{ background: #1F1C19; border-radius: 10px 10px 0 0; padding: 12px 18px; max-width: 560px; margin: 0 auto; display: flex; align-items: center; gap: 10px; }}
  .subject-tag {{ font-size: 9px; letter-spacing: 0.18em; text-transform: uppercase; color: #6A5F54; flex-shrink: 0; }}
  .subject-text {{ font-size: 12.5px; color: #FAF7F3; font-weight: 500; letter-spacing: 0.01em; }}
  .subject-text em {{ color: #C4714A; font-style: normal; }}
  .email-header {{ position: relative; width: 100%; height: 200px; background-size: cover; background-position: center; background-repeat: no-repeat; }}
  .email-header-overlay {{ position: absolute; inset: 0; background: linear-gradient(to bottom, rgba(0,0,0,0.1) 0%, rgba(0,0,0,0.4) 100%); display: flex; align-items: center; justify-content: space-between; padding: 22px 36px; }}
  .header-logo {{ display: flex; align-items: center; gap: 10px; }}
  .logo-icon {{ width: 34px; height: 34px; background: #C4714A; border-radius: 9px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; }}
  .logo-name {{ font-family: 'Cormorant Garamond', serif; font-size: 21px; font-weight: 600; color: #FFFFFF; letter-spacing: 0.03em; line-height: 1; }}
  .logo-name em {{ font-style: italic; color: #C4714A; }}
  .header-urgent {{ display: flex; align-items: center; gap: 6px; background: rgba(255, 240, 232, 0.95); border: 1px solid #F0C4A0; padding: 5px 11px; border-radius: 20px; }}
  .urgent-dot {{ width: 6px; height: 6px; background: #C4714A; border-radius: 50%; }}
  .urgent-text {{ font-size: 9px; letter-spacing: 0.18em; text-transform: uppercase; color: #C4714A; font-weight: 500; }}
  .urgency-band {{ background: #C4714A; padding: 12px 36px; display: flex; align-items: center; justify-content: space-between; }}
  .urgency-band p {{ font-size: 11.5px; color: rgba(255,255,255,0.9); font-weight: 400; }}
  .urgency-band strong {{ color: white; font-weight: 500; }}
  .urgency-timer {{ font-family: 'Cormorant Garamond', serif; font-size: 13px; color: white; font-weight: 600; background: rgba(0,0,0,0.15); padding: 4px 10px; border-radius: 6px; white-space: nowrap; }}
  .email-body {{ padding: 32px 36px; }}
  .greeting {{ font-size: 15px; font-weight: 500; color: #1A1814; margin-bottom: 12px; }}
  .body-text {{ font-size: 13.5px; color: #5A5248; line-height: 1.75; margin-bottom: 20px; }}
  .lead-card {{ background: #FAF7F3; border-radius: 16px; border: 1px solid #EDE8E0; overflow: hidden; margin: 24px 0; }}
  .lead-card-header {{ background: #1A1814; padding: 14px 20px; display: flex; align-items: center; justify-content: space-between; }}
  .lead-card-header span {{ font-size: 9px; letter-spacing: 0.22em; text-transform: uppercase; color: rgba(255,255,255,0.5); }}
  .lead-id {{ font-family: 'Cormorant Garamond', serif; font-size: 13px; color: #C4714A; font-weight: 600; letter-spacing: 0.1em; }}
  .lead-card-body {{ padding: 20px; }}
  .lead-row {{ display: flex; align-items: flex-start; gap: 12px; padding: 10px 0; border-bottom: 1px solid #EDE8E0; }}
  .lead-row:last-child {{ border-bottom: none; }}
  .lead-icon {{ width: 30px; height: 30px; background: #F0E0D4; border-radius: 8px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; margin-top: 1px; }}
  .lead-detail {{ }}
  .lead-detail .key {{ font-size: 9px; letter-spacing: 0.2em; text-transform: uppercase; color: #B0A090; margin-bottom: 2px; }}
  .lead-detail .val {{ font-size: 13.5px; font-weight: 500; color: #1A1814; line-height: 1.3; }}
  .lead-detail .val.budget {{ font-family: 'Cormorant Garamond', serif; font-size: 20px; font-weight: 600; color: #C4714A; }}
  .competition-strip {{ background: #FFF8F0; border: 1px solid #F0D4B8; border-radius: 12px; padding: 14px 18px; margin: 20px 0; display: flex; align-items: center; gap: 12px; }}
  .comp-icon {{ width: 36px; height: 36px; background: #F0D4B8; border-radius: 10px; display: flex; align-items: center; justify-content: center; flex-shrink: 0; font-size: 18px; }}
  .comp-text {{ font-size: 12.5px; color: #7A4820; line-height: 1.55; }}
  .comp-text strong {{ color: #C4714A; font-weight: 600; }}
  .intent-note {{ background: #F4F9F5; border-left: 3px solid #5A9A6A; border-radius: 0 10px 10px 0; padding: 14px 16px; margin: 16px 0; }}
  .intent-note p {{ font-size: 12px; color: #3A6A48; line-height: 1.6; }}
  .intent-note strong {{ font-weight: 600; }}
  .cta-wrap {{ margin: 28px 0 8px; text-align: center; }}
  .cta-btn-main {{ display: inline-block; background: #C4714A; color: white; text-decoration: none; font-family: 'DM Sans', sans-serif; font-size: 14px; font-weight: 500; letter-spacing: 0.05em; padding: 16px 40px; border-radius: 13px; width: 100%; text-align: center; box-sizing: border-box; }}
  .cta-sub {{ font-size: 11px; color: #B0A090; text-align: center; margin-top: 10px; }}
  .tips-row {{ display: flex; gap: 10px; margin: 20px 0; }}
  .tip-item {{ flex: 1; background: #FAF7F3; border-radius: 12px; padding: 14px; border: 1px solid #EDE8E0; text-align: center; }}
  .tip-item .tip-icon {{ font-size: 20px; margin-bottom: 6px; }}
  .tip-item .tip-text {{ font-size: 10.5px; color: #7A6F64; line-height: 1.5; }}
  .tip-item .tip-text strong {{ display: block; font-size: 11px; color: #1A1814; margin-bottom: 2px; }}
  .email-divider {{ height: 1px; background: #EDE8E0; margin: 24px 0; }}
  .email-footer {{ background: #161412; padding: 28px 36px; }}
  .footer-top {{ display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px; }}
  .footer-logo-name {{ font-family: 'Cormorant Garamond', serif; font-size: 18px; font-weight: 600; color: #FAF7F3; }}
  .footer-logo-name em {{ font-style: italic; color: #C4714A; }}
  .footer-text {{ font-size: 11px; color: #5A5048; line-height: 1.7; margin-bottom: 16px; }}
  .footer-links {{ display: flex; gap: 18px; flex-wrap: wrap; margin-bottom: 16px; }}
  .footer-links a {{ font-size: 10px; color: #7A6F64; text-decoration: none; letter-spacing: 0.06em; }}
  .footer-divider {{ height: 1px; background: #2A2520; margin-bottom: 14px; }}
  .footer-legal {{ font-size: 10px; color: #3A3028; line-height: 1.6; }}
</style>
</head>
<body>
  <div class="subject-line">
    <span class="subject-tag">Subject</span>
    <span class="subject-text">
      🔔 New rental request in <em>{seeker_location}</em> — {budget} budget, move-in {move_in_timeline}
    </span>
  </div>

  <div class="email-shell">
    <div class="email-header" style="background-image: url('{header_url}');">
      <div class="email-header-overlay">
        <div class="header-logo">
          <div class="logo-icon">
            <svg width="19" height="19" viewBox="0 0 24 24" fill="none">
              <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
              <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
            </svg>
          </div>
          <span class="logo-name">Veri<em>nest</em></span>
        </div>
        <div class="header-urgent">
          <div class="urgent-dot"></div>
          <span class="urgent-text">New Lead</span>
        </div>
      </div>
    </div>

    <div class="urgency-band">
      <p><strong>3 other agents</strong> in your area have received this lead</p>
      <span class="urgency-timer">⏱ Respond first</span>
    </div>

    <div class="email-body">
      <p class="greeting">Hi {agent_name},</p>
      <p class="body-text">A serious seeker just posted a rental request on Verinest that matches your area. Here are their full requirements — respond now to be first agent they hear from.</p>

      <div class="lead-card">
        <div class="lead-card-header">
          <span>Seeker Request Details</span>
          <span class="lead-id">{lead_id}</span>
        </div>
        <div class="lead-card-body">
          <div class="lead-row">
            <div class="lead-icon">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
                <path d="M12 2C8.13 2 5 5.13 5 9c0 5.25 7 13 7 13s7-7.75 7-13c0-3.87-3.13-7-7-7z" stroke="#C4714A" stroke-width="2" fill="none"/>
                <circle cx="12" cy="9" r="2.5" stroke="#C4714A" stroke-width="2" fill="none"/>
              </svg>
            </div>
            <div class="lead-detail">
              <div class="key">Location</div>
              <div class="val">{seeker_location}</div>
            </div>
          </div>

          <div class="lead-row">
            <div class="lead-icon">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
                <path d="M3 11L12 3l9 8v9a1 1 0 01-1 1H5a1 1 0 01-1-1v-9z" stroke="#C4714A" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
                <path d="M9 21V12h6v9" stroke="#C4714A" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div class="lead-detail">
              <div class="key">Property Type</div>
              <div class="val">{property_type}</div>
            </div>
          </div>

          <div class="lead-row">
            <div class="lead-icon">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="9" stroke="#C4714A" stroke-width="2"/>
                <path d="M12 7v5l3 3" stroke="#C4714A" stroke-width="2" stroke-linecap="round"/>
              </svg>
            </div>
            <div class="lead-detail">
              <div class="key">Move-In Timeline</div>
              <div class="val">{move_in_timeline}</div>
            </div>
          </div>

          <div class="lead-row">
            <div class="lead-icon">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
                <rect x="2" y="5" width="20" height="14" rx="2" stroke="#C4714A" stroke-width="2" fill="none"/>
                <path d="M2 10h20" stroke="#C4714A" stroke-width="2"/>
              </svg>
            </div>
            <div class="lead-detail">
              <div class="key">Annual Budget</div>
              <div class="val budget">{budget}</div>
            </div>
          </div>

          <div class="lead-row">
            <div class="lead-icon">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
                <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2" stroke="#C4714A" stroke-width="2" stroke-linecap="round"/>
                <circle cx="9" cy="7" r="4" stroke="#C4714A" stroke-width="2"/>
              </svg>
            </div>
            <div class="lead-detail">
              <div class="key">Seeker Type</div>
              <div class="val">{seeker_type}</div>
            </div>
          </div>

          <div class="lead-row">
            <div class="lead-icon">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
                <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" stroke="#C4714A" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
              </svg>
            </div>
            <div class="lead-detail">
              <div class="key">Additional Notes</div>
              <div class="val" style="font-size:13px;color:#5A5248;font-weight:400;">"{additional_notes}"</div>
            </div>
          </div>
        </div>
      </div>

      <div class="intent-note">
        <p>✓ <strong>Verified seeker intent:</strong> This request was submitted with a confirmed phone number and email address. The seeker has acknowledged our terms and is actively looking — not browsing casually.</p>
      </div>

      <div class="competition-strip">
        <div class="comp-icon">⚡</div>
        <div class="comp-text">
          <strong>3 other verified agents in your area have received this lead.</strong> The first agent to respond with a matching property gets priority placement in this seeker's dashboard. Don't let another agent take it.
        </div>
      </div>

      <div class="cta-wrap">
        <a href="{action_url}" class="cta-btn-main">Respond to This Lead →</a>
        <p class="cta-sub">Takes less than 2 minutes · Log in to send your offer</p>
      </div>

      <div class="email-divider"></div>

      <div class="tips-row">
        <div class="tip-item">
          <div class="tip-icon">📸</div>
          <div class="tip-text">
            <strong>Add photos</strong>
            Listings with photos get 4x more responses
          </div>
        </div>
        <div class="tip-item">
          <div class="tip-icon">💬</div>
          <div class="tip-text">
            <strong>Write a note</strong>
            Personal messages close faster than generic offers
          </div>
        </div>
        <div class="tip-item">
          <div class="tip-icon">📅</div>
          <div class="tip-text">
            <strong>Offer a date</strong>
            Suggest a specific viewing date immediately
          </div>
        </div>
      </div>

      <p class="body-text" style="font-size:12px;color:#9A8F84;margin-top:8px;">This lead will expire in <strong style="color:#C4714A;">48 hours</strong> if no agent responds. After that it is re-broadcast to a wider pool of agents.</p>
    </div>

    <div class="email-footer">
      <div class="footer-top">
        <span class="footer-logo-name">Veri<em>nest</em></span>
        <div class="logo-icon" style="width:28px;height:28px;border-radius:8px;">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none">
            <path d="M3 11L12 3l9 8" stroke="white" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
            <rect x="9" y="13" width="6" height="8" rx="3" fill="white"/>
          </svg>
        </div>
      </div>
      <p class="footer-text">You're receiving this because you're a verified agent on Verinest covering your area. Lead notifications are sent in real time when seekers post needs in your area.</p>
      <div class="footer-links">
        <a href="#">My Dashboard</a>
        <a href="#">Active Leads</a>
        <a href="#">My Listings</a>
        <a href="#">Agent Support</a>
        <a href="#">Notification Settings</a>
      </div>
      <div class="footer-divider"></div>
      <p class="footer-legal">© 2026 Verinest, Nigeria · verinest.ng<br>To stop receiving lead alerts, update your notification preferences.</p>
    </div>
  </div>
</body>
</html>"#,
                header_url = header_url,
                agent_name = agent_name,
                seeker_location = seeker_location,
                property_type = property_type,
                budget = budget,
                move_in_timeline = move_in_timeline,
                seeker_name = seeker_name,
                seeker_type = seeker_type,
                additional_notes = additional_notes,
                lead_id = lead_id,
                action_url = action_url
            ),
        }
    }
}

pub const EMAIL_VERIFICATION_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en" xmlns:v="urn:schemas-microsoft-com:vml">
<head>
  <meta charset="utf-8">
  <meta name="x-apple-disable-message-reformatting">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="format-detection" content="telephone=no, date=no, address=no, email=no, url=no">
  <meta name="color-scheme" content="light dark">
  <meta name="supported-color-schemes" content="light dark">
  <!--[if mso]>
  <noscript>
    <xml>
      <o:OfficeDocumentSettings xmlns:o="urn:schemas-microsoft-com:office:office">
        <o:PixelsPerInch>96</o:PixelsPerInch>
      </o:OfficeDocumentSettings>
    </xml>
  </noscript>
  <style>
    td,th,div,p,a,h1,h2,h3,h4,h5,h6 {font-family: "Segoe UI", sans-serif; mso-line-height-rule: exactly;}
  </style>
  <![endif]-->
  <title>Email verification</title>
  <style>
    img {
      max-width: 100%;
      vertical-align: middle
    }
    @media (max-width: 600px) {
      .sm-px-4 {
        padding-left: 16px !important;
        padding-right: 16px !important
      }
      .sm-px-6 {
        padding-left: 24px !important;
        padding-right: 24px !important
      }
      .sm-leading-8 {
        line-height: 32px !important
      }
    }
  </style>
</head>
<body style="margin: 0; width: 100%; background-color: #15b786; padding: 0; -webkit-font-smoothing: antialiased; word-break: break-word">
  <div style="display: none">
    Please use this code to verify your email.
    &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847;
  </div>
  <div role="article" aria-roledescription="email" aria-label="Email verification" lang="en">
    <div class="sm-px-4" style="margin-top: 200px; margin-bottom: 200px; font-family: ui-sans-serif, system-ui, -apple-system, 'Segoe UI', sans-serif">
      <table align="center" cellpadding="0" cellspacing="0" role="none">
        <tr>
          <td style="width: 552px; max-width: 100%">
            <table style="width: 100%;" cellpadding="0" cellspacing="0" role="none">
              <tr>
                <td class="sm-px-6" style="border-radius: 4px; background-color: #fffffe; padding: 48px; text-align: center; font-size: 16px; color: #334155; box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05)">
                  <h1 class="sm-leading-8" style="margin: 0 0 24px; font-size: 24px; font-weight: 600; color: #000001">
                    Email Verification
                  </h1>
                  <p style="margin: 0; line-height: 24px">
                    Please use this code to verify your email.
                  </p>
                  <div role="separator" style="line-height: 24px">&zwj;</div>
                  <div style="background-color: #e5e7eb; padding: 8px 4px">
                    <p style="font-size: 16px; font-weight: 600;">{{ verification_code }}</p>
                  </div>
                </td>
              </tr>
            </table>
          </td>
        </tr>
      </table>
    </div>
  </div>
</body>
</html>
"#;

pub const WILDMETA_EMAIL_VERIFICATION_TEMPLATE: &str = r##"
<!DOCTYPE html>
<html lang="en" xmlns:v="urn:schemas-microsoft-com:vml">
<head>
  <meta charset="utf-8">
  <meta name="x-apple-disable-message-reformatting">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="format-detection" content="telephone=no, date=no, address=no, email=no, url=no">
  <meta name="color-scheme" content="light dark">
  <meta name="supported-color-schemes" content="light dark">
  <!--[if mso]>
  <noscript>
    <xml>
      <o:OfficeDocumentSettings xmlns:o="urn:schemas-microsoft-com:office:office">
        <o:PixelsPerInch>96</o:PixelsPerInch>
      </o:OfficeDocumentSettings>
    </xml>
  </noscript>
  <style>
    td,th,div,p,a,h1,h2,h3,h4,h5,h6 {font-family: "Segoe UI", sans-serif; mso-line-height-rule: exactly;}
  </style>
  <![endif]-->
  <title>Email Verification - Wildmeta</title>
  <style>
    .from-black {
      --tw-gradient-from: #000001 var(--tw-gradient-from-position) !important;
      --tw-gradient-to: rgb(0 0 1 / 0) var(--tw-gradient-to-position) !important;
      --tw-gradient-stops: var(--tw-gradient-from), var(--tw-gradient-to) !important
    }
    .to-gray-900 {
      --tw-gradient-to: #111827 var(--tw-gradient-to-position) !important
    }
    .hover-text-gray-700:hover {
      color: #374151 !important
    }
    @media (max-width: 600px) {
      .sm-ml-3 {
        margin-left: 12px !important
      }
      .sm-block {
        display: block !important
      }
      .sm-px-16 {
        padding-left: 64px !important;
        padding-right: 64px !important
      }
    }
  </style>
</head>
<body style="margin: 0; width: 100%; background-color: #f9fafb; padding: 0; -webkit-font-smoothing: antialiased; word-break: break-word">
  <div style="display: none">
    Please verify your email address to complete your registration.
    &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847; &#8199;&#65279;&#847;
  </div>
  <div role="article" aria-roledescription="email" aria-label="Email Verification - Wildmeta" lang="en">
    <table cellpadding="0" cellspacing="0" style="width: 100%;" role="none">
      <tr>
        <td style="background-color: #000001">
          <div class="from-black to-gray-900" style="position: relative; background-image: linear-gradient(to bottom right, #000001 var(--tw-gradient-from-position), #111827 var(--tw-gradient-to-position))">
            <div style="position: relative; padding: 48px 24px; text-align: center">
              <table width="100%" cellpadding="0" cellspacing="0" role="none">
                <tr>
                  <td align="center" style="padding-bottom: 24px">
                    <img src="https://dex-cdn.wildmeta.io/index/email_wildmetalogo.png" alt="Wildmeta" width="150" height="48" border="0" style="max-width: 100%; vertical-align: middle; display: block; width: 150px; height: 48px; margin: 0 auto">
                  </td>
                </tr>
              </table>
              <table width="100%" cellpadding="0" cellspacing="0" role="none">
                <tr>
                  <td align="center" style="padding-bottom: 20px">
                    <div style="font-size: 36px; font-weight: 300">
                      <span class="sm-block" style="color: #E3FFFE; display: inline">Trade
                      better on</span>
                      <span class="sm-block sm-ml-3" style="display: inline">
                      <img src="https://dex-cdn.wildmeta.io/index/email_hyperliquid.png" alt="Hyperliquid" width="160" height="35" border="0" style="max-width: 100%; display: inline-block; width: 160px; height: 35px; vertical-align: middle;">
                    </span>
                    </div>
                  </td>
                </tr>
              </table> <!--[if !mso]><!-->
              <table width="100%" cellpadding="0" cellspacing="0" role="none">
                <tr>
                  <td align="center" style="padding-top: 20px">
                    <div style="display: inline-block">
                      <span style="padding-left: 16px; padding-right: 16px; font-size: 14px; font-weight: 500; color: #d1d5db">Discover</span>
                      <span style="font-size: 14px; color: #6b7280">|</span>
                      <span style="padding-left: 16px; padding-right: 16px; font-size: 14px; font-weight: 500; color: #d1d5db;">Learn</span>
                      <span style="font-size: 14px; color: #6b7280;">|</span>
                      <span style="padding-left: 16px; padding-right: 16px; font-size: 14px; font-weight: 500; color: #d1d5db;">Trade</span>
                    </div>
                  </td>
                </tr>
              </table>
              <!--<![endif]-->
            </div>
          </div>
        </td>
      </tr>
    </table>
    <table cellpadding="0" cellspacing="0" style="width: 100%;" role="none">
      <tr>
        <td style="position: relative; background-color: #fffffe">
          <div style="position: relative; z-index: 10; padding: 64px 24px 32px; text-align: center">
            <table width="100%" cellpadding="0" cellspacing="0" role="none">
              <tr>
                <td align="center" style="padding-bottom: 40px">
                  <p style="margin: 0 auto; max-width: 256px; font-size: 18px; font-weight: 400; line-height: 28px; color: #111827">
                    You are requesting email sign up, your verification code is:
                  </p>
                </td>
              </tr>
            </table>
            <table width="100%" cellpadding="0" cellspacing="0" role="none">
              <tr>
                <td align="center" style="padding-bottom: 32px;">
                  <div class="sm-px-16" style="display: inline-block; border-radius: 16px; background-color: #f3f4f6; padding: 24px 48px">
                    <div style="font-family: ui-monospace, Menlo, Consolas, monospace; font-size: 36px; font-weight: 700; line-height: 1; letter-spacing: 0.1em; color: #111827">
                       {{ verification_code }}
                    </div>
                  </div>
                </td>
              </tr>
            </table>
            <table width="100%" cellpadding="0" cellspacing="0" role="none">
              <tr>
                <td align="center">
                  <div style="display: inline-block;"> <a href="https://x.com/wildmetahq" target="_blank" rel="noopener noreferrer" class="hover-text-gray-700" style="display: inline-block; padding-left: 16px; padding-right: 16px; color: #6b7280;">
                      <img src="https://dex-cdn.wildmeta.io/index/x.png" alt="X (Twitter)" width="28" height="28" border="0" style="max-width: 100%; vertical-align: middle; display: block; width: 28px; height: 28px;">
                    </a> <a href="https://discord.com/invite/fQ6QMuNpnj" target="_blank" rel="noopener noreferrer" class="hover-text-gray-700" style="display: inline-block; padding-left: 16px; padding-right: 16px; color: #6b7280;">
                      <img src="https://dex-cdn.wildmeta.io/index/discord.png" alt="Discord" width="30" height="30" border="0" style="max-width: 100%; vertical-align: middle; display: block; width: 30px; height: 30px;">
                    </a>
                  </div>
                </td>
              </tr>
            </table>
          </div>
        </td>
      </tr>
    </table>
    <table cellpadding="0" cellspacing="0" style="width: 100%;" role="none">
      <tr>
        <td style="background-color: #fffffe; padding-left: 24px; padding-right: 24px; padding-bottom: 48px">
          <table width="100%" cellpadding="0" cellspacing="0" style="margin-left: auto; margin-right: auto; max-width: 384px" role="none">
            <tr>
              <td style="padding-bottom: 24px; text-align: center;">
                <p style="margin: 0; font-size: 14px; line-height: 24px; color: #4b5563">
                  Thank you for choosing Wildmeta. If you need any assistance, please contact our official team using the
                  methods above. We are committed to providing top-quality service and addressing all your inquiries.
                </p>
              </td>
            </tr>
            <tr>
              <td style="padding-bottom: 24px; text-align: center;">
                <p style="margin: 0 0 12px; font-size: 12px; line-height: 20px; color: #6b7280">
                  <strong>Risk warning:</strong> Digital asset prices can be volatile. The value of your investment may go
                  down or up and you may not get back the amount invested. You are solely responsible for your investment
                  decisions and Wildmeta is not liable for any losses you may incur. <strong>Not financial
                  advice.</strong> For more information, see our
                  <a href="https://docs.wildmeta.io/terms-of-service" target="_blank" rel="noopener noreferrer" style="color: #179B90; text-decoration: underline;">Terms of Use</a> and
                  <a href="https://docs.wildmeta.io/privacy" target="_blank" rel="noopener noreferrer" style="color: #179B90; text-decoration: underline;">Privacy Policy</a>.
                </p>
                <p style="margin: 0; font-size: 12px; line-height: 20px; color: #6b7280;">
                  <strong>Kindly note:</strong> Please be aware of phishing sites and always make sure you are visiting
                  the official Wildmeta.ai website when entering sensitive data.
                </p>
              </td>
            </tr>
            <tr>
              <td style="border-top-width: 1px; border-color: #e5e7eb; padding-top: 24px; text-align: center">
                <p style="margin: 0; font-size: 12px; color: #6b7280;">
                  © 2025 Wildmeta, All Rights Reserved.
                </p>
              </td>
            </tr>
          </table>
        </td>
      </tr>
    </table>
  </div>
</body>
</html>
"##;

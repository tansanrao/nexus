#!/bin/sh
set -e

echo "🔐 Configuring HTTP Basic Authentication..."

# Check if authentication credentials are provided
if [ -n "$HTTP_BASIC_AUTH_USERNAME" ] && [ -n "$HTTP_BASIC_AUTH_PASSWORD" ]; then
    echo "   ✓ Auth credentials found"

    # Generate .htpasswd file using htpasswd (from apache2-utils)
    # -c creates the file, -b uses the password from command line
    htpasswd -cb /etc/nginx/.htpasswd "$HTTP_BASIC_AUTH_USERNAME" "$HTTP_BASIC_AUTH_PASSWORD"

    echo "   ✓ Generated .htpasswd file"
    echo "   ✓ HTTP Basic Auth enabled for user: $HTTP_BASIC_AUTH_USERNAME"
else
    echo "   ⚠ Warning: HTTP_BASIC_AUTH_USERNAME or HTTP_BASIC_AUTH_PASSWORD not set"
    echo "   ⚠ Running without authentication - NOT RECOMMENDED FOR PRODUCTION"

    # Create an empty .htpasswd file to prevent nginx errors
    touch /etc/nginx/.htpasswd
fi

# Configure API proxy target
API_PROXY_TARGET=${API_PROXY_TARGET:-http://api-server:8000}
echo "🔌 Configuring API proxy..."
echo "   ✓ API Proxy Target: $API_PROXY_TARGET"

# Replace environment variable in nginx config
sed -i "s|\${API_PROXY_TARGET}|$API_PROXY_TARGET|g" /etc/nginx/conf.d/default.conf

echo "🚀 Starting nginx..."

# Execute the main container command (nginx)
exec "$@"


FROM node:20-alpine
RUN npm install -g supergateway @modelcontextprotocol/server-filesystem
EXPOSE 3000
ENTRYPOINT ["sh", "-c", "\
  mkdir -p /workspace && \
  exec supergateway \
    --stdio 'npx -y @modelcontextprotocol/server-filesystem /workspace' \
    --outputTransport streamableHttp \
    --port 3000 \
    --stateful \
    --healthEndpoint /healthz"]

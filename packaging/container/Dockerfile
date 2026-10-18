# Phygital Key A2A as a container (ghcr.io/phygital-key/phygital-key-a2a).
# Built by .github/workflows/release.yml from the same Linux binaries the signed
# release contains (their SHA-256 is in that release's signed release.txt).
#
#   docker run -d --name phygital-key-a2a -p 127.0.0.1:7478:7478 \
#     -v phygital-key-a2a:/data ghcr.io/phygital-key/phygital-key-a2a:latest
#   docker exec -it phygital-key-a2a phygital-key-a2a enroll
#   docker exec -it phygital-key-a2a phygital-key-a2a token
#
# Publish the port on 127.0.0.1 only, or put a TLS proxy in front and pass
# --origin https://your.host. See docs/own-server.md.
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f
ARG TARGETARCH
LABEL org.opencontainers.image.title="Phygital Key A2A" \
      org.opencontainers.image.description="A person approves exact AI actions with a passkey; any service checks the signed yes offline." \
      org.opencontainers.image.source="https://github.com/Phygital-Key/phygital-key-a2a" \
      org.opencontainers.image.licenses="Apache-2.0"
COPY --chmod=0755 bin/${TARGETARCH}/phygital-key-a2a /usr/local/bin/phygital-key-a2a
COPY --chown=65532:65532 data /data
COPY NOTICE LICENSE /usr/share/doc/phygital-key-a2a/
USER 65532:65532
ENV PHYGITAL_KEY_A2A_HOME=/data
VOLUME ["/data"]
EXPOSE 7478
ENTRYPOINT ["/usr/local/bin/phygital-key-a2a"]
# Inside the container the gate must listen on all addresses so Docker can reach it.
CMD ["--bind", "0.0.0.0", "serve"]

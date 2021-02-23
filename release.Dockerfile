FROM centos:centos8
RUN mkdir /app
WORKDIR /app
COPY build_target/release/dating_questions_bot .
ENTRYPOINT ["./dating_questions_bot"]
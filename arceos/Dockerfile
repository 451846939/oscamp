FROM arceos-rust-base:nightly-2024-08-27

RUN ln -sf /usr/share/zoneinfo/Asia/Shanghai /etc/localtime \
    && echo "Asia/Shanghai" > /etc/timezone \

# 可选加构建依赖等
WORKDIR /project
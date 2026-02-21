#!/bin/bash
# Sea Lantern PPA - gh CLI 便捷命令脚本
# 用法: source scripts/ppa-gh-commands.sh

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Sea Lantern PPA - gh CLI 命令工具${NC}"
echo ""

# 触发 PPA 构建
ppa_build() {
    local version=${1:-"0.6.5"}
    echo -e "${YELLOW}触发 PPA 构建 - 版本: $version${NC}"
    gh workflow run build-and-upload-ppa.yml -f version="$version"
    echo -e "${GREEN}✅ 工作流已触发${NC}"
    echo "查看状态: gh run list --workflow=build-and-upload-ppa.yml -L 5"
}

# 触发 PPA 测试
ppa_test() {
    local version=${1:-"0.6.5"}
    local distro=${2:-"noble"}
    echo -e "${YELLOW}触发 PPA 测试 - 版本: $version, 发行版: $distro${NC}"
    gh workflow run ppa-test.yml -f version="$version" -f distro="$distro"
    echo -e "${GREEN}✅ 测试工作流已触发${NC}"
    echo "查看状态: gh run list --workflow=ppa-test.yml -L 5"
}

# 查看 PPA 构建状态
ppa_status() {
    echo -e "${BLUE}PPA 工作流运行状态:${NC}"
    echo ""
    echo -e "${YELLOW}构建工作流:${NC}"
    gh run list --workflow=build-and-upload-ppa.yml -L 5
    echo ""
    echo -e "${YELLOW}测试工作流:${NC}"
    gh run list --workflow=ppa-test.yml -L 5
}

# 查看最近的构建日志
ppa_logs() {
    local workflow=${1:-"build-and-upload-ppa"}
    echo -e "${YELLOW}获取 $workflow 的最新日志...${NC}"
    local run_id=$(gh run list --workflow="$workflow.yml" -L 1 --json databaseId -q '.[0].databaseId')
    if [ -n "$run_id" ]; then
        gh run view "$run_id" --log
    else
        echo -e "${RED}❌ 没有找到运行记录${NC}"
    fi
}

# 查询 Launchpad PPA 状态
ppa_lp_status() {
    echo -e "${BLUE}Launchpad PPA 状态:${NC}"
    echo ""

    # PPA 基本信息
    echo -e "${YELLOW}PPA 信息:${NC}"
    curl -s "https://api.launchpad.net/1.0/~brianeee7878/+archive/ubuntu/sealantern" | \
        python3 -c "import sys,json; d=json.load(sys.stdin); \
print(f\"  名称: {d.get('displayname')}\"); \
print(f\"  签名指纹: {d.get('signing_key_fingerprint')}\"); \
print(f\"  仓库格式: {d.get('repository_format')}\")"

    echo ""
    echo -e "${YELLOW}已发布的源码包:${NC}"
    curl -s "https://api.launchpad.net/1.0/~brianeee7878/+archive/ubuntu/sealantern?ws.op=getPublishedSources" | \
        python3 -c "import sys,json; d=json.load(sys.stdin); \
entries = [e for e in d.get('entries',[]) if e.get('status')=='Published']; \
[print(f\"  ✅ {e.get('display_name')}\") for e in entries[:3]]"

    echo ""
    echo -e "${YELLOW}最近的构建记录:${NC}"
    curl -s "https://api.launchpad.net/1.0/~brianeee7878/+archive/ubuntu/sealantern?ws.op=getBuildRecords" | \
        python3 -c "import sys,json; d=json.load(sys.stdin); \
[print(f\"  {e.get('buildstate')[:1]} {e.get('title')[:60]}...\") for e in d.get('entries',[])][:5]"
}

# 下载并测试本地 deb 包
ppa_download_test() {
    local version=${1:-"0.6.5"}
    local build_num=${2:-"23"}
    local distro=${3:-"noble"}

    echo -e "${YELLOW}下载 PPA 构建的 deb 包...${NC}"

    local deb_url="https://launchpad.net/~brianeee7878/+archive/ubuntu/sealantern/+build/32304679/+files/sea-lantern-ppa-updater_${version}~${build_num}~${distro}1_amd64.deb"
    local temp_deb="/tmp/sea-lantern-ppa-updater_test.deb"

    wget -q "$deb_url" -O "$temp_deb"

    if [ -f "$temp_deb" ]; then
        echo -e "${GREEN}✅ 下载完成${NC}"
        echo ""
        echo -e "${YELLOW}包信息:${NC}"
        dpkg-deb -I "$temp_deb"
        echo ""
        echo -e "${YELLOW}包内容:${NC}"
        dpkg-deb -c "$temp_deb" | head -20
        echo ""
        echo -e "${BLUE}要安装测试，请运行:${NC}"
        echo "  sudo dpkg -i $temp_deb"
    else
        echo -e "${RED}❌ 下载失败${NC}"
    fi
}

# 显示帮助
ppa_help() {
    echo -e "${BLUE}Sea Lantern PPA - 可用命令:${NC}"
    echo ""
    echo -e "${GREEN}ppa_build [版本]${NC}"
    echo "  触发 PPA 构建工作流"
    echo "  示例: ppa_build 0.6.5"
    echo ""
    echo -e "${GREEN}ppa_test [版本] [发行版]${NC}"
    echo "  触发 PPA 测试工作流"
    echo "  示例: ppa_test 0.6.5 noble"
    echo ""
    echo -e "${GREEN}ppa_status${NC}"
    echo "  查看 PPA 工作流运行状态"
    echo ""
    echo -e "${GREEN}ppa_logs [工作流名]${NC}"
    echo "  查看最近的工作流日志"
    echo "  示例: ppa_logs build-and-upload-ppa"
    echo ""
    echo -e "${GREEN}ppa_lp_status${NC}"
    echo "  查询 Launchpad PPA 状态"
    echo ""
    echo -e "${GREEN}ppa_download_test [版本] [构建号] [发行版]${NC}"
    echo "  下载并查看 PPA 构建的 deb 包"
    echo "  示例: ppa_download_test 0.6.5 23 noble"
    echo ""
}

# 显示帮助信息
ppa_help

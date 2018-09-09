pipeline {
    agent {
        dockerfile {
            filename 'Dockerfile'
            dir 'jenkins'
            reuseNode false
        }
    }

    stages {
        stage('Cleanup') {
            steps {
                sh """
git clean -fdx
"""
            }
        }
        stage('Build (Linux)') {
            steps {
                sh """
cargo test
"""
            }
        }
        stage('Build (Windows 32-bit)') {
            steps {
                sh """
export RUSTFLAGS="-C panic=abort"
cargo build --target i686-pc-windows-gnu
"""
            }
        }
        stage('Build (Windows 64-bit)') {
            steps {
                sh """
cargo build --target x86_64-pc-windows-gnu
"""
            }
        }
    }
}

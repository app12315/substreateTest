FROM paritytech/substrate-playground-template-base:sha-0793587

USER root
RUN apt-get update && apt-get install -y zsh wget
 
RUN wget https://github.com/robbyrussell/oh-my-zsh/raw/master/tools/install.sh -O - | zsh || true 
USER substrate
CMD ["zsh"] 